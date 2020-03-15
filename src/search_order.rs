use core::{Core, WORDLISTS};
use exception::{INVALID_NUMERIC_ARGUMENT, RESULT_OUT_OF_RANGE, SEARCH_ORDER_OVERFLOW};

pub trait SearchOrder: Core {
    fn add_search_order(&mut self) {
        self.add_primitive("get-current", SearchOrder::get_current);
        self.add_primitive("set-current", SearchOrder::set_current);
        self.add_primitive("definitions", SearchOrder::definitions);
        self.add_primitive("get-order", SearchOrder::get_order);
        self.add_primitive("set-order", SearchOrder::set_order);
        self.add_primitive("wordlist", SearchOrder::p_wordlist);
    }

    /// ( -- wid )
    ///
    primitive! {fn p_wordlist(&mut self) {
        let next_wordlist = self.wordlist().last_wordlist  + 1;
        if next_wordlist < WORDLISTS {
            self.wordlist_mut().last_wordlist = next_wordlist;
            self.s_stack().push(next_wordlist as _);
        } else {
            self.abort_with(RESULT_OUT_OF_RANGE);
        }
    }}

    /// ( -- wid )
    ///
    /// Return wid, the identifier of the compilation word list.
    primitive! {fn get_current(&mut self) {
        let current = self.wordlist().current;
        self.s_stack().push(current as _);
    }}

    /// ( wid -- )
    ///
    /// Set the compilation word list to the word list identified by wid.
    primitive! {fn set_current(&mut self) {
        let current = self.s_stack().pop() as _;
        self.wordlist_mut().current = current;
    }}

    /// ( -- )
    ///
    /// Make the compilation word list the same as the first word list in
    /// the search order. Specifies that the names of subsequent definitions
    /// will be placed in the compilation word list. Subsequent changes in
    /// the search order will not affect the compilation word list.
    primitive! {fn definitions(&mut self) {
        let search_order_len = self.wordlist().search_order_len;
        if search_order_len > 0 {
            let current = self.wordlist().search_order[search_order_len-1];
            self.wordlist_mut().current = current;
        } else {
            // Make the compilation wordlist to be FORTH.
            self.wordlist_mut().current = 0;
        }
    }}

    /// ( -- widn ... wid1 n )
    /// Returns the number of word lists n in the search order and the word
    /// list identifiers widn ... wid1 identifying these word lists. wid1
    /// identifies the word list that is searched first, and widn the word
    /// list that is searched last. The search order is unaffected.
    primitive! {fn get_order(&mut self) {
        let search_order_len = self.wordlist().search_order_len;
        let search_order: [usize; WORDLISTS] = self.wordlist().search_order;
        for i in 0..search_order_len {
            self.s_stack().push(search_order[i] as _);
        }
        self.s_stack().push(search_order_len as _);
    }}

    /// ( widn ... wid1 n -- )
    /// Set the search order to the word lists identified by widn ... wid1.
    /// Subsequently, word list wid1 will be searched first, and word list
    /// widn searched last. If n is zero, empty the search order. If n is minus
    /// one, set the search order to the implementation-defined minimum search
    /// order. The minimum search order shall include the words FORTH-WORDLIST
    /// and SET-ORDER. A system shall allow n to be at least eight.
    primitive! {fn set_order(&mut self) {
        let search_order_len = self.s_stack().pop() as usize;
        if search_order_len > WORDLISTS {
            self.abort_with(SEARCH_ORDER_OVERFLOW);
        } else {
            for i in (0..search_order_len).rev() {
                let wid = self.s_stack().pop() as _;
                if wid >= WORDLISTS {
                    self.abort_with(INVALID_NUMERIC_ARGUMENT);
                } else {
                    self.wordlist_mut().search_order[i] = wid;
                }
            }
            self.wordlist_mut().search_order_len = search_order_len;
        }
    }}
}

mod tests {
    use core::{Core, WORDLISTS};
    use exception::{RESULT_OUT_OF_RANGE, UNDEFINED_WORD};
    use mock_vm::VM;

    #[test]
    fn test_wordlist() {
        let vm = &mut VM::new(16, 16);
        for i in 1..WORDLISTS {
            vm.set_source("WORDLIST");
            vm.evaluate_input();
            assert_eq!(vm.last_error(), None);
            assert_eq!(vm.s_stack().pop(), i as _);
        }
        vm.set_source("WORDLIST");
        vm.evaluate_input();
        assert_eq!(vm.last_error(), Some(RESULT_OUT_OF_RANGE));
    }

    #[test]
    fn test_current_and_search_order() {
        let vm = &mut VM::new(16, 16);
        vm.set_source("WORDLIST");
        vm.evaluate_input();
        let _ = vm.s_stack().pop();
        vm.set_source("GET-CURRENT");
        vm.evaluate_input();
        assert_eq!(vm.s_stack().pop(), 0);
        vm.set_source("0 CONSTANT ITEM");
        vm.evaluate_input();
        assert_eq!(vm.last_error(), None);
        vm.set_source("1 SET-CURRENT 1 CONSTANT ITEM");
        vm.evaluate_input();
        assert_eq!(vm.last_error(), None);
        vm.set_source("0 1 2 SET-ORDER");
        vm.evaluate_input();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.s_stack().as_slice(), []);
        vm.set_source("GET-ORDER");
        vm.evaluate_input();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.s_stack().as_slice(), [0, 1, 2]);
        let _ = vm.s_stack().pop3();
        vm.set_source("0 1 2 SET-ORDER ITEM");
        vm.evaluate_input();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.s_stack().as_slice(), [1]);
        let _ = vm.s_stack().pop();
        vm.set_source("1 0 2 SET-ORDER ITEM");
        vm.evaluate_input();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.s_stack().as_slice(), [0]);
        let _ = vm.s_stack().pop();

        // DEFIIITIONS
        vm.set_source("1 0 2 SET-ORDER DEFINITIONS 0 CONSTANT ZERO");
        vm.evaluate_input();
        vm.set_source("0 1 2 SET-ORDER DEFINITIONS 1 CONSTANT ONE");
        vm.evaluate_input();
        vm.set_source("0 1 SET-ORDER   ' ZERO DROP");
        vm.evaluate_input();
        assert_eq!(vm.last_error(), None);
        vm.set_source("0 1 SET-ORDER   ' ONE DROP");
        vm.evaluate_input();
        assert_eq!(vm.last_error(), Some(UNDEFINED_WORD));
        vm.set_source("0 1 2 SET-ORDER   ' ONE DROP");
        vm.evaluate_input();
        assert_eq!(vm.last_error(), None);
    }
}
