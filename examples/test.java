package Test;

class Main {
    static native void out(int i);

    public static void main(String []args) {
        for (int i = 0; i < 10; i++) {
            out(i);
        }
    }
}
