package Test;

class Main {
    static int i = 0;
    static native void out(int i);

    public static int run() {
        return 5;
    }

    public static void main(String []args) {
        int a[] = new int[2];
        a[0] = run();
        a[1] = 4;

        for (int i = 0; i < a.length; i++) {
            out(a[i]);
        }
    }
}
