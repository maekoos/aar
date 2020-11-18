// import java.lang.Math;

public class MyCode extends Code {  
  private int getMax2(int a) {
    // int b = this.r123();
    // int c = this.r123();
    //todo: System.out.println("Hello world!");

    int[] myNum = {1, 10, 20, 30, 40};

    Code c = new Code();
    myNum[0]= c.r123(1, 2, 3, 4);
    
    
    myNum[1]= c.test(1);
    
    
    return myNum[0] + myNum[1];
    // return Math.abs(myNum[0]) + myNum[1];


    // if (a == myNum[0]) {
    //   a += 1;
    //   return 9999;
    // }
    // switch(a) {
    //   case 10:
    //     return 11;
    //   case 11:
    //     return 11;
    //   case 13:
    //     return 11;
    //   case 14:
    //     return 13;
    //   case 15:
    //     return 13;
    //   case 16:
    //     return 13;
    //   case 170:
    //     return 13;
    //   default:
    //     return 0;
    // }
  }
}

class Code {
  public int r123(int a, int b, int c, int d) {
    return a + b + c + d;
  }

  public static int test(int a) {
    return a + a;
  }
}