use std::{cell::RefCell, rc::Rc};

use astro::transceiver::Transceiver;

struct UavSim {
    tc: Rc<RefCell<Transceiver>>,
}