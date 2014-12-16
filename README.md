Toy lisp
========

This is an attempt at a very simple Lisp (or maybe Scheme, not sure yet).
Currently the only working things are '+', 'def' and ints. Examples:

	>>> (def foo (+ 1 2 3 4))
	result: Ok(Int(10))
	>>> (+ foo 1)
	result: Ok(Int(11))

...so it's a barely functional calculator ATM. I learned a whole lot about
Rust, though, especially about mixing boxes with values/guards/cells/etc.
Thanks to #rust for that.
