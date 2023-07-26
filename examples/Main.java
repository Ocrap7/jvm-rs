package Test;

class Main {
    static int i = 0;
    static native void out(int i);

    public static void main(String []args) {

        for (int i = 0; i < 10; i++) {
            other(i);
        }
    }

    public static void other(int i) {
        out(i + 2);
    }
}
