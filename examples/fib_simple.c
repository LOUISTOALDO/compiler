int fib(int n) {
    if (n <= 1) {
        return n;
    }
    return fib(n - 1);
}

int main() {
    return fib(5);
}
