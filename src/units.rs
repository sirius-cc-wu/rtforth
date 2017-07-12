use std::f64::consts::PI;
use core::Core;

pub trait Units: Core {
    fn add_units(&mut self) {

        self.add_primitive("meter", Units::from_meter);
        self.add_primitive("mm", Units::from_mm);
        self.add_primitive("um", Units::from_um);

        self.add_primitive("deg", Units::from_deg);
        self.add_primitive("rad", Units::from_rad);

        self.add_primitive("min", Units::from_minute);
        self.add_primitive("sec", Units::from_sec);
        self.add_primitive("msec", Units::from_msec);
        self.add_primitive("usec", Units::from_usec);

    }

    primitive!{fn from_meter(&mut self) {
        let t = self.f_stack().pop();
        self.f_stack().push(t);
    }}

    primitive!{fn from_mm(&mut self) {
        let t = self.f_stack().pop();
        self.f_stack().push(t*0.001);
    }}

    primitive!{fn from_um(&mut self) {
        let t = self.f_stack().pop();
        self.f_stack().push(t*0.000_001);
    }}

    primitive!{fn from_deg(&mut self) {
        let t = self.f_stack().pop();
        self.f_stack().push(t*PI/180.0);
    }}

    primitive!{fn from_rad(&mut self) {
        let t = self.f_stack().pop();
        self.f_stack().push(t);
    }}

    primitive!{fn from_minute(&mut self) {
        let t = self.f_stack().pop();
        self.f_stack().push(t*60.0);
    }}

    primitive!{fn from_sec(&mut self) {
        let t = self.f_stack().pop();
        self.f_stack().push(t);
    }}

    primitive!{fn from_msec(&mut self) {
        let t = self.f_stack().pop();
        self.f_stack().push(t*0.001);
    }}

    primitive!{fn from_usec(&mut self) {
        let t = self.f_stack().pop();
        self.f_stack().push(t*0.000_001);
    }}
}

#[cfg(test)]
mod tests {
    use vm::VM;
    use core::Core;

    fn double_value_check(res: f64, exp: f64) -> bool {
        if (res > exp - 0.000_000_1) && (res < exp + 0.000_000_1) {
            return true;
        }
        false
    }

    #[test]
    fn test_units_meter() {
        let vm = &mut VM::new(16, 16);
        vm.set_source("0.1234 meter");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.f_stack().len(), 1);
        let t = vm.f_stack().pop();
        assert!(double_value_check(t, 0.1234));
    }


    #[test]
    fn test_units_mm() {
        let vm = &mut VM::new(16, 16);
        vm.set_source("0.3 mm");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.f_stack().len(), 1);
        let t = vm.f_stack().pop();
        assert!(double_value_check(t, 0.000_3));
    }

    #[test]
    fn test_units_um() {
        let vm = &mut VM::new(16, 16);
        vm.set_source("3.0 um");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.f_stack().len(), 1);
        let t = vm.f_stack().pop();
        assert!(double_value_check(t, 0.000_003));
    }


    #[test]
    fn test_units_deg() {
        let vm = &mut VM::new(16, 16);
        vm.set_source("10.0 deg");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.f_stack().len(), 1);
        let t = vm.f_stack().pop();
        assert!(double_value_check(t, 0.174_532_9));
    }

    #[test]
    fn test_units_rad() {
        let vm = &mut VM::new(16, 16);
        vm.set_source("10.0 rad");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.f_stack().len(), 1);
        let t = vm.f_stack().pop();
        assert!(double_value_check(t, 10.0));
    }

    #[test]
    fn test_units_min() {
        let vm = &mut VM::new(16, 16);
        vm.set_source("1.0 min");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.f_stack().len(), 1);
        let t = vm.f_stack().pop();
        assert!(double_value_check(t, 60.0));
    }

    #[test]
    fn test_units_sec() {
        let vm = &mut VM::new(16, 16);
        vm.set_source("2.0 sec");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.f_stack().len(), 1);
        let t = vm.f_stack().pop();
        assert!(double_value_check(t, 2.0));
    }

    #[test]
    fn test_units_msec() {
        let vm = &mut VM::new(16, 16);
        vm.set_source("2.0 msec");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.f_stack().len(), 1);
        let t = vm.f_stack().pop();
        assert!(double_value_check(t, 0.002));
    }

    #[test]
    fn test_units_usec() {
        let vm = &mut VM::new(16, 16);
        vm.set_source("2.0 usec");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.f_stack().len(), 1);
        let t = vm.f_stack().pop();
        assert!(double_value_check(t, 0.000_002));
    }

}
