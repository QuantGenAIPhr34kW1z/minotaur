use crate::ffi::MinotaurOutput;
use anyhow::Result;
use std::fs::File;
use std::io::{BufWriter, Write};

pub struct CsvWriter {
    w: BufWriter<File>,
}

impl CsvWriter {
    pub fn create(path: &str) -> Result<Self> {
        let f = File::create(path)?;
        Ok(Self { w: BufWriter::new(f) })
    }

    pub fn write_header(&mut self) -> Result<()> {
        writeln!(
            self.w,
            "case,bpr,opr,mach,alt_km,status,converged,iter,mass_resid,energy_resid,final_residual,final_bpr,t4,tsfc_proxy,thrust_proxy"
        )?;
        Ok(())
    }

    pub fn write_row(
        &mut self,
        case: &str,
        bpr: f64,
        opr: f64,
        mach: f64,
        alt_km: f64,
        out: &MinotaurOutput,
    ) -> Result<()> {
        let converged = if out.status == 0 { "true" } else { "false" };
        writeln!(
            self.w,
            "{},{:.6},{:.6},{:.4},{:.4},{},{},{},{:.6e},{:.6e},{:.6e},{:.6},{:.2},{:.6},{:.6}",
            case,
            bpr,
            opr,
            mach,
            alt_km,
            out.status,
            converged,
            out.iter,
            out.mass_resid,
            out.energy_resid,
            out.final_residual,
            out.final_bpr,
            out.t4,
            out.tsfc_proxy,
            out.thrust_proxy
        )?;
        Ok(())
    }

    pub fn flush(&mut self) -> Result<()> {
        self.w.flush()?;
        Ok(())
    }
}
