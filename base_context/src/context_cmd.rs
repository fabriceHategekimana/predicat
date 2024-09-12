use parser::cmd::Cmd;
use crate::simple_context::SimpleContext;


pub trait ContextCMD {
    fn get_aftercmds(&self) -> Vec<Cmd>;
    fn add_aftercmd(self, aftcmd: &[Cmd]) -> Self;
}


impl ContextCMD for SimpleContext {
    fn add_aftercmd(self, aftcmd: &[Cmd]) -> SimpleContext {
        SimpleContext{
           tab: self.tab,
           cmds: self.cmds.iter().chain(aftcmd.iter()).map(|x| x.clone()).collect(),
           log: self.log
        }
    }

    fn get_aftercmds(&self) -> Vec<Cmd> {
        self.cmds.clone()
    }
}
