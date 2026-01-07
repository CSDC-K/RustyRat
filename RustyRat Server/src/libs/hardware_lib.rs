

use hardware_query::{GPUVendor, HardwareInfo};
pub fn get_hardware() -> String {
    let hw = HardwareInfo::query().unwrap();


    let cpu = hw.cpu();
    let ram = hw.memory();
 
    let mut gpu_vendor: Option<GPUVendor> = None;
    let mut gpu_modelname: Option<String> = None;
    let mut gpu_gb: Option<f64> = None;

    for gpu in hw.gpus() {
        gpu_vendor = Some(gpu.vendor.clone());
        gpu_modelname = Some(gpu.model_name.clone());
        gpu_gb = Some(gpu.memory_gb().clone());

        break;
    }


    let raw_fingerprint = format!(
        "cpu_name:{}|
        cpu_logicores:{}|
        cpu_basefreq:{}|
        cpu_pyhscore:{}|
        ram_channels:{}|
        ram_eccsupport:{}|
        ram_banwith:{}|
        ram_speed:{}|
        ram_usage:{}|
        gpu_vendor:{:?}|
        gpu_modelname:{:?}|
        gpu_gb:{:?}|",


        cpu.model_name,
        cpu.logical_cores,
        cpu.base_frequency,
        cpu.physical_cores,
        ram.channels,
        ram.ecc_support,
        ram.total_mb,
        ram.speed_mhz,
        ram.used_mb,
        gpu_vendor,
        gpu_modelname,
        gpu_gb,    
    );

    raw_fingerprint

}
