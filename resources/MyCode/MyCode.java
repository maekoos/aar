// import java.lang.Math;

public class MyCode extends Code {  
  int b = 10;
  
  private int getMax2(int a) {
    // int b = this.r123();
    // int c = this.r123();
    //todo: System.out.println("Hello world!");

    // int[] myNums = {1, 10, 20, 30, 40};

    Code c = new Code();
    // int myNum0 = c.r123(this.a, 2, 3, 4);
    // int myNum1 = c.test(this.b);
    
    
    return this.a + this.b + c.a;
  }
}

class Code {
  int a = 5;
  
  public int r123(int a, int b, int c, int d) {
    return a + b + c + d;
  }

  public static int test(int a) {
    return a + a;
  }
}