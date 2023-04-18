use anyhow::Result;
use futures::{
    future::{BoxFuture, LocalBoxFuture},
    stream::FuturesUnordered,
    Future, FutureExt, StreamExt,
};
use std::{pin::Pin, sync::Arc};

mod module_loader;

fn create_web_worker_callback(
    _ps: deno_cli::proc_state::ProcState,
    _stdio: deno_runtime::deno_io::Stdio,
) -> Arc<deno_runtime::ops::worker_host::CreateWebWorkerCb> {
    // TODO: Implement this based on https://github.com/denoland/deno/blob/d07aa4a0723b04583b7cb1e09152457d866d13d3/cli/worker.rs#L643
    Arc::new(move |_args| todo!("Web worker support"))
}

fn create_web_worker_preload_module_callback() -> Arc<deno_runtime::ops::worker_host::WorkerEventCb>
{
    Arc::new(move |worker| {
        let fut = async move { Ok(worker) };
        futures::task::LocalFutureObj::new(Box::new(fut))
    })
}

fn create_web_worker_pre_execute_module_callback(
    ps: deno_cli::proc_state::ProcState,
) -> Arc<deno_runtime::ops::worker_host::WorkerEventCb> {
    Arc::new(move |mut worker| {
        let ps = ps.clone();
        let fut = async move {
            // this will be up to date after pre-load
            if ps.npm_resolver.has_packages() {
                deno_runtime::deno_node::initialize_runtime(
                    &mut worker.js_runtime,
                    ps.options.has_node_modules_dir(),
                    None,
                )?;
            }

            Ok(worker)
        };
        futures::task::LocalFutureObj::new(Box::new(fut))
    })
}

// TODO: Add deno ops here, could for example completely isolate API calls & prevent leaking the token entirely
fn make_windmill_deno_exts() -> Vec<deno_runtime::deno_core::Extension> {
    vec![]
}

// Adapted from https://github.com/denoland/deno/blob/d07aa4a0723b04583b7cb1e09152457d866d13d3/cli/worker.rs#L437 with modifications (primarily removing non-deno entrypoint)
async fn create_main_worker(
    ps: &deno_cli::proc_state::ProcState,
    main_module: deno_core::url::Url,
    permissions: deno_runtime::permissions::PermissionsContainer,
    stdio: deno_runtime::deno_io::Stdio,
) -> Result<(deno_core::url::Url, deno_runtime::worker::MainWorker)> {
    let mut custom_extensions: Vec<deno_runtime::deno_core::Extension> = make_windmill_deno_exts();

    let module_loader = module_loader::WindmillModuleLoader::new(
        ps.clone(),
        deno_runtime::permissions::PermissionsContainer::allow_all(),
        permissions.clone(),
    );

    let maybe_inspector_server = ps.maybe_inspector_server.clone();

    let create_web_worker_cb = create_web_worker_callback(ps.clone(), stdio.clone());
    let web_worker_preload_module_cb = create_web_worker_preload_module_callback();
    let web_worker_pre_execute_module_cb =
        create_web_worker_pre_execute_module_callback(ps.clone());

    let maybe_storage_key = ps.options.resolve_storage_key(&main_module);
    let origin_storage_dir = maybe_storage_key.as_ref().map(|key| {
        ps.dir
            .origin_data_folder_path()
            .join(deno_cli::util::checksum::gen(&[key.as_bytes()]))
    });
    let cache_storage_dir = maybe_storage_key.map(|key| {
        // DENO_TODO(@satyarohith): storage quota management
        // Note: we currently use temp_dir() to avoid managing storage size.
        std::env::temp_dir()
            .join("deno_cache")
            .join(deno_cli::util::checksum::gen(&[key.as_bytes()]))
    });

    let mut extensions = Vec::new();
    // extensions.append(&mut deno_cli::ops::cli_exts(ps.clone()));
    extensions.append(&mut custom_extensions);

    let options = deno_runtime::worker::WorkerOptions {
        bootstrap: deno_runtime::BootstrapOptions {
            args: ps.options.argv().clone(),
            cpu_count: 1,
            debug_flag: false,
            enable_testing_features: ps.options.enable_testing_features(),
            locale: "en".to_owned(),
            location: None,
            no_color: true,
            is_tty: false,
            runtime_version: deno_cli::version::deno().to_string(),
            ts_version: deno_cli::version::TYPESCRIPT.to_string(),
            unstable: true,
            user_agent: format!(
                "Windmill/{}; {}",
                env!("CARGO_PKG_VERSION"),
                deno_cli::version::get_user_agent()
            ),
            inspect: false,
        },
        extensions,
        startup_snapshot: Some(deno_cli::js::deno_isolate_init()),
        unsafely_ignore_certificate_errors: None,
        root_cert_store: Some(ps.root_cert_store.clone()),
        seed: None,
        source_map_getter: Some(Box::new(module_loader.clone())),
        format_js_error_fn: Some(Arc::new(deno_runtime::fmt_errors::format_js_error)),
        create_web_worker_cb,
        web_worker_preload_module_cb,
        web_worker_pre_execute_module_cb,
        maybe_inspector_server,
        should_break_on_first_statement: false,
        should_wait_for_inspector_session: false,
        module_loader,
        npm_resolver: Some(std::rc::Rc::new(ps.npm_resolver.clone())),
        get_error_class_fn: Some(&deno_cli::errors::get_error_class_name),
        cache_storage_dir,
        origin_storage_dir,
        blob_store: ps.blob_store.clone(),
        broadcast_channel: ps.broadcast_channel.clone(),
        shared_array_buffer_store: Some(ps.shared_array_buffer_store.clone()),
        compiled_wasm_module_store: Some(ps.compiled_wasm_module_store.clone()),
        stdio,
    };

    let worker = deno_runtime::worker::MainWorker::bootstrap_from_options(
        main_module.clone(),
        permissions,
        options,
    );

    Ok((main_module, worker))
}

async fn run_main(
    main_module: &deno_core::url::Url,
    worker: &mut deno_runtime::worker::MainWorker,
    ps: &deno_cli::proc_state::ProcState,
) -> Result<i32> {
    let id = worker.preload_main_module(&main_module).await?;
    if ps.npm_resolver.has_packages() || ps.graph().has_node_specifier {
        deno_runtime::deno_node::initialize_runtime(&mut worker.js_runtime, false, None)?;
    }
    worker.evaluate_module(id).await?;

    worker.dispatch_load_event(deno_core::located_script_name!())?;

    loop {
        worker.run_event_loop(false).await?;
        if !worker.dispatch_beforeunload_event(deno_core::located_script_name!())? {
            break;
        }
    }

    worker.dispatch_unload_event(deno_core::located_script_name!())?;

    Ok(worker.exit_code())
}

fn make_cli_options(
    flags: deno_cli::args::Flags,
    job_dir: &str,
) -> Result<deno_cli::args::CliOptions> {
    let opts = deno_cli::args::CliOptions::new(flags, job_dir.into(), None, None, None)?;

    Ok(opts)
}

async fn run_deno_cli(
    args: Vec<String>,
    job_dir: &str,
    cache_dir: &str,
) -> std::result::Result<i32, anyhow::Error> {
    let mut flags = deno_cli::args::flags_from_vec(args)
        .expect("Args are built by the app and should always be valid");

    flags.cache_path = Some(cache_dir.into());

    deno_cli::util::v8::init_v8_flags(&flags.v8_flags, deno_cli::util::v8::get_v8_flags_from_env());

    let _ = tracing_log::LogTracer::init(); // TODO: I don't think this works. Not really what we want anyways
                                            // deno_cli::util::logger::init(flags.log_level);

    let deno_cli::args::flags::DenoSubcommand::Run(run_flags) = flags.subcommand.clone() else {
        unreachable!("Flags should always be set to run");
    };

    let ps =
        deno_cli::proc_state::ProcState::from_options(Arc::new(make_cli_options(flags, job_dir)?))
            .await?;

    let main_module = deno_core::resolve_url_or_path(&run_flags.script, ps.options.initial_cwd())
        .map_err(deno_core::error::AnyError::from)?;

    let permissions = deno_runtime::permissions::PermissionsContainer::new(
        deno_runtime::permissions::Permissions::from_options(&ps.options.permissions_options())?,
    );
    // TODO: Handle log streaming here
    // This may require either streaming through a file (which is ugly)
    // or changing a bit of code in deno to use streams internally (given this is in deno_runtime, this maybe hard. Investigate.)
    let stdio = deno_runtime::deno_io::Stdio {
        stdin: deno_runtime::deno_io::StdioPipe::Inherit,
        stdout: deno_runtime::deno_io::StdioPipe::Inherit,
        stderr: deno_runtime::deno_io::StdioPipe::Inherit,
    };

    let (main_module, mut worker) =
        create_main_worker(&ps, main_module, permissions, stdio).await?;

    let exit_code = run_main(&main_module, &mut worker, &ps).await?;

    Ok(exit_code)
}

#[derive(Debug)]
struct Params {
    args: Vec<String>,
    job_dir: String,
    cache_dir: String,
    notification: tokio::sync::oneshot::Sender<Result<i32, anyhow::Error>>,
}

fn run_multi(
    killpill: tokio::sync::broadcast::Receiver<()>,
) -> tokio::sync::mpsc::UnboundedSender<Params> {
    let killpill: &'static mut tokio::sync::broadcast::Receiver<()> = Box::leak(Box::new(killpill));
    let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
    let handle = tokio::runtime::Handle::current();
    std::thread::spawn(move || {
        handle.block_on(CustomFut(
            receiver,
            FuturesUnordered::new(),
            KillpillStatus::Active(Box::pin(killpill.recv())),
        ));
    });

    sender
}

pub fn setup(killpill: tokio::sync::broadcast::Receiver<()>) {
    SENDER.set(run_multi(killpill)).unwrap()
}

pub fn enqueue(
    args: Vec<String>,
    job_dir: String,
    cache_dir: String,
    notification: tokio::sync::oneshot::Sender<Result<i32, anyhow::Error>>,
) {
    SENDER
        .get()
        .expect("setup should be called")
        .send(Params { args, job_dir, cache_dir, notification })
        .unwrap();
}

static SENDER: once_cell::sync::OnceCell<tokio::sync::mpsc::UnboundedSender<Params>> =
    once_cell::sync::OnceCell::new();

struct CustomFut<'a>(
    tokio::sync::mpsc::UnboundedReceiver<Params>,
    FuturesUnordered<LocalBoxFuture<'a, ()>>,
    KillpillStatus,
);

enum KillpillStatus {
    Active(Pin<Box<dyn Future<Output = Result<(), tokio::sync::broadcast::error::RecvError>>>>),
    Shutdown,
}

impl<'a> Future for CustomFut<'a> {
    type Output = ();

    fn poll(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let m = self.get_mut();
        match &mut m.2 {
            KillpillStatus::Active(f) => {
                if let std::task::Poll::Ready(_) = f.poll_unpin(cx) {
                    m.2 = KillpillStatus::Shutdown;
                    return std::task::Poll::Ready(()); // TODO: Handle shutdown better somehow...
                }
            }
            KillpillStatus::Shutdown => {
                if let std::task::Poll::Ready(_) = m.1.poll_next_unpin(cx) {
                    return std::task::Poll::Ready(());
                }
                return std::task::Poll::Pending;
            }
        }

        match m.0.poll_recv(cx) {
            std::task::Poll::Ready(Some(new_params)) => {
                m.1.push(Box::pin(async move {
                    let res =
                        run_deno_cli(new_params.args, &new_params.job_dir, &new_params.cache_dir)
                            .await;
                    new_params.notification.send(res).unwrap();
                }));
            }
            std::task::Poll::Ready(None) => {
                println!("Dropped connection?!");
                return std::task::Poll::Ready(());
            }
            std::task::Poll::Pending => (),
        };

        let _ = m.1.poll_next_unpin(cx);

        std::task::Poll::Pending
    }
}
