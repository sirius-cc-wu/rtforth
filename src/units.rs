use core::Core;
use std::f64::consts::PI;

const RPM: f64 = 2.0 * PI / 60.0;

pub trait Units: Core {
    fn add_units(&mut self) {
        self.add_primitive("meter", Units::from_meter);
        self.add_primitive("mm", Units::from_mm);
        self.add_primitive("um", Units::from_um);

        self.add_primitive("deg", Units::from_deg);
        self.add_primitive("rad", Units::from_rad);

        self.add_primitive("hr", Units::from_hour);
        self.add_primitive("minute", Units::from_minute);
        self.add_primitive("sec", Units::from_sec);
        self.add_primitive("msec", Units::from_msec);
        self.add_primitive("usec", Units::from_usec);

        self.add_primitive("mm/min", Units::mm_per_min);
        self.add_primitive("mm/sec", Units::mm_per_sec);
        self.add_primitive("um/msec", Units::um_per_msec);

        self.add_primitive("rpm", Units::rpm);
        self.add_primitive("hz", Units::hertz);
        self.add_primitive("1/sec", Units::hertz);
    }

    primitive! {fn from_meter(&mut self) {
        // Do nothing.
    }}

    primitive! {fn from_mm(&mut self) {
        let t = self.f_stack().pop();
        self.f_stack().push(0.001 * t);
    }}

    primitive! {fn from_um(&mut self) {
        let t = self.f_stack().pop();
        self.f_stack().push(0.000001 * t);
    }}

    primitive! {fn from_deg(&mut self) {
        let t = self.f_stack().pop();
        self.f_stack().push(t*PI/180.0);
    }}

    primitive! {fn from_rad(&mut self) {
        // Do nothing.
    }}

    primitive! {fn from_hour(&mut self) {
        let t = self.f_stack().pop();
        self.f_stack().push(60.0 * 60.0 * t);
    }}

    primitive! {fn from_minute(&mut self) {
        let t = self.f_stack().pop();
        self.f_stack().push(60.0 * t);
    }}

    primitive! {fn from_sec(&mut self) {
        // Do nothing.
    }}

    primitive! {fn from_msec(&mut self) {
        let t = self.f_stack().pop();
        self.f_stack().push(0.001 * t);
    }}

    primitive! {fn from_usec(&mut self) {
        let t = self.f_stack().pop();
        self.f_stack().push(0.000001 * t);
    }}

    primitive! {fn mm_per_min(&mut self) {
        let t = self.f_stack().pop();
        self.f_stack().push(0.001/60.0 * t);
    }}

    primitive! {fn mm_per_sec(&mut self) {
        let t = self.f_stack().pop();
        self.f_stack().push(0.001 * t);
    }}

    primitive! {fn um_per_msec(&mut self) {
        let t = self.f_stack().pop();
        self.f_stack().push(0.000001/0.001 * t);
    }}

    primitive! {fn rpm(&mut self) {
        let t = self.f_stack().pop();
        self.f_stack().push(t*RPM);
    }}

    primitive! {fn hertz(&mut self) {
        let t = self.f_stack().pop();
        self.f_stack().push(t);
    }}
}

#[cfg(test)]
mod tests {
    use core::Core;
    use mock_vm::VM;
    use std::f64::consts::PI;

    fn double_value_check(res: f64, exp: f64) -> bool {
        if (res > exp - 0.000_000_1) && (res < exp + 0.000_000_1) {
            return true;
        }
        false
    }

    #[test]
    fn test_units_meter() {
        let vm = &mut VM::new();
        vm.set_source("0.1234E meter");
        vm.evaluate_input();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.f_stack().len(), 1);
        let t = vm.f_stack().pop();
        assert!(double_value_check(t, 0.1234));
    }

    #[test]
    fn test_units_mm() {
        let vm = &mut VM::new();
        vm.set_source("0.3E mm");
        vm.evaluate_input();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.f_stack().len(), 1);
        let t = vm.f_stack().pop();
        assert!(double_value_check(t, 0.000_3));
    }

    #[test]
    fn test_units_um() {
        let vm = &mut VM::new();
        vm.set_source("3.0E um");
        vm.evaluate_input();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.f_stack().len(), 1);
        let t = vm.f_stack().pop();
        assert!(double_value_check(t, 0.000_003));
    }

    #[test]
    fn test_units_deg() {
        let vm = &mut VM::new();
        vm.set_source("10.0E deg");
        vm.evaluate_input();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.f_stack().len(), 1);
        let t = vm.f_stack().pop();
        assert!(double_value_check(t, 0.174_532_9));
    }

    #[test]
    fn test_units_rad() {
        let vm = &mut VM::new();
        vm.set_source("10.0E rad");
        vm.evaluate_input();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.f_stack().len(), 1);
        let t = vm.f_stack().pop();
        assert!(double_value_check(t, 10.0));
    }

    #[test]
    fn test_units_hr() {
        let vm = &mut VM::new();
        vm.set_source("1.0E hr");
        vm.evaluate_input();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.f_stack().len(), 1);
        let t = vm.f_stack().pop();
        assert!(double_value_check(t, 60.0 * 60.0));
    }

    #[test]
    fn test_units_minute() {
        let vm = &mut VM::new();
        vm.set_source("1.0E minute");
        vm.evaluate_input();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.f_stack().len(), 1);
        let t = vm.f_stack().pop();
        assert!(double_value_check(t, 60.0));
    }

    #[test]
    fn test_units_sec() {
        let vm = &mut VM::new();
        vm.set_source("2.0E sec");
        vm.evaluate_input();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.f_stack().len(), 1);
        let t = vm.f_stack().pop();
        assert!(double_value_check(t, 2.0));
    }

    #[test]
    fn test_units_msec() {
        let vm = &mut VM::new();
        vm.set_source("2.0E msec");
        vm.evaluate_input();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.f_stack().len(), 1);
        let t = vm.f_stack().pop();
        assert!(double_value_check(t, 0.002));
    }

    #[test]
    fn test_units_usec() {
        let vm = &mut VM::new();
        vm.set_source("2.0E usec");
        vm.evaluate_input();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.f_stack().len(), 1);
        let t = vm.f_stack().pop();
        assert!(double_value_check(t, 0.000_002));
    }

    #[test]
    fn test_units_mm_per_min() {
        let vm = &mut VM::new();
        vm.set_source("2.0E mm/min");
        vm.evaluate_input();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.f_stack().len(), 1);
        let t = vm.f_stack().pop();
        assert!(double_value_check(t, 0.002 / 60.0));
    }

    #[test]
    fn test_units_um_per_msec() {
        let vm = &mut VM::new();
        vm.set_source("2.0E um/msec");
        vm.evaluate_input();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.f_stack().len(), 1);
        let t = vm.f_stack().pop();
        assert!(double_value_check(t, 0.000002 / 0.001));
    }

    #[test]
    fn test_rpm() {
        let vm = &mut VM::new();
        vm.set_source("2.0E rpm");
        vm.evaluate_input();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.f_stack().len(), 1);
        let t = vm.f_stack().pop();
        assert!(double_value_check(t, 2.0 * 2.0 * PI / 60.0));
    }

    #[test]
    fn test_hz() {
        let vm = &mut VM::new();
        vm.set_source("2.0E hz");
        vm.evaluate_input();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.f_stack().len(), 1);
        let t = vm.f_stack().pop();
        assert!(double_value_check(t, 2.0));
    }
}
