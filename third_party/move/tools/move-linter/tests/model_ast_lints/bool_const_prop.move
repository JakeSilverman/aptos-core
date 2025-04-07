module 0xc0ffee::m {
    const TRUE: bool = true;

    fun foo(x: u64): bool {
        x > 10
    }


    spec schema Foo {
        x : bool;
        y : u64;
        ensures x && true && y >= 0;
    }

    fun take(_x: bool) {}

    fun bar() {}

    public fun test1(x: u64) {
        if (true && foo(x) && true) { bar() };
    }

}

module 0xc0ffee::no_warn {
    #[lint::skip(unnecessary_boolean_identity_comparison)]
    public fun test(jakejake: bool) {
        if (jakejake == true) abort 1;
    }

    spec schema Test {
        jakejake : bool;
        ensures jakejake && true;
    }
}
