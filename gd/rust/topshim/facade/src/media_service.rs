//! Media service facade

use bt_topshim::btif::BluetoothInterface;
use bt_topshim::profiles::a2dp::{A2dp, A2dpCallbacksDispatcher, A2dpSink};
use bt_topshim::profiles::avrcp::{Avrcp, AvrcpCallbacksDispatcher};
use bt_topshim_facade_protobuf::facade::{
    A2dpSourceConnectRequest, A2dpSourceConnectResponse, StartA2dpRequest, StartA2dpResponse,
};
use bt_topshim_facade_protobuf::facade_grpc::{create_media_service, MediaService};

use grpcio::*;

use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;

fn get_a2dp_dispatcher() -> A2dpCallbacksDispatcher {
    A2dpCallbacksDispatcher { dispatch: Box::new(move |_cb| {}) }
}

fn get_avrcp_dispatcher() -> AvrcpCallbacksDispatcher {
    AvrcpCallbacksDispatcher { dispatch: Box::new(move |_cb| {}) }
}

/// Main object for Media facade service
#[derive(Clone)]
pub struct MediaServiceImpl {
    rt: Arc<Runtime>,
    pub btif_a2dp: Arc<Mutex<A2dp>>,
    btif_a2dp_sink: Arc<Mutex<A2dpSink>>,
    pub btif_avrcp: Arc<Mutex<Avrcp>>,
}

impl MediaServiceImpl {
    /// Create a new instance of the root facade service
    pub fn create(rt: Arc<Runtime>, btif_intf: Arc<Mutex<BluetoothInterface>>) -> grpcio::Service {
        let mut btif_a2dp = A2dp::new(&btif_intf.lock().unwrap());
        let btif_a2dp_sink = A2dpSink::new(&btif_intf.lock().unwrap());
        let mut btif_avrcp = Avrcp::new(&btif_intf.lock().unwrap());
        btif_a2dp.initialize(get_a2dp_dispatcher());
        btif_avrcp.initialize(get_avrcp_dispatcher());

        create_media_service(Self {
            rt,
            btif_a2dp: Arc::new(Mutex::new(btif_a2dp)),
            btif_a2dp_sink: Arc::new(Mutex::new(btif_a2dp_sink)),
            btif_avrcp: Arc::new(Mutex::new(btif_avrcp)),
        })
    }
}

impl MediaService for MediaServiceImpl {
    fn start_a2dp(
        &mut self,
        ctx: RpcContext<'_>,
        req: StartA2dpRequest,
        sink: UnarySink<StartA2dpResponse>,
    ) {
        if req.start_a2dp_source {
            ctx.spawn(async move {
                sink.success(StartA2dpResponse::default()).await.unwrap();
            })
        } else if req.start_a2dp_sink {
            self.btif_a2dp_sink.lock().unwrap().initialize();
            ctx.spawn(async move {
                sink.success(StartA2dpResponse::default()).await.unwrap();
            })
        }
    }

    fn a2dp_source_connect(
        &mut self,
        ctx: RpcContext<'_>,
        req: A2dpSourceConnectRequest,
        sink: UnarySink<A2dpSourceConnectResponse>,
    ) {
        let a2dp = self.btif_a2dp.clone();
        ctx.spawn(async move {
            a2dp.lock().unwrap().connect(req.address.clone());
            a2dp.lock().unwrap().set_active_device(req.address.clone());
            sink.success(A2dpSourceConnectResponse::default()).await.unwrap();
        })
    }
}
