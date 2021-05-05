class MyCode {
  public static void main(String[] args) {
    int a = 10;
    try {
      a /= 0;
      int c = 100;
      System.out.println(c);
    } catch(ArithmeticException err) {
      System.out.println("Caught exception ;)");
      System.out.println(err); // java.lang.ArithmeticException: / by zero
    }

    try {
      int b = a % 0;
    } catch(ArithmeticException err) {
      System.out.println("err");
      System.out.println(err);
    } catch(ArrayIndexOutOfBoundsException err) {
      System.out.println("Java index exception");
    }

    System.out.println(a);
  }
}