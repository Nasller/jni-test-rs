import java.util.Comparator;
import java.util.ArrayList;
import java.util.List;

public class App{
  static {
    System.loadLibrary("target/debug/jni_test_rs");
  }

  public native static String helloJNI() throws Exception;

  /**
    * native方法 : 返回类的实例对象
    * @param targetClass 需要查询实例的Class
    * @param limitNum 最多返回实例的数量
    * @return
    */
  public static native Object[] getInstances(Class<?> targetClass,int limitNum) throws Exception;

  /**
    * 获取类的所有的实例对象
    * @param targetClass 需要查询实例的Class
    * @return
    */
  public static Object[] getInstances(Class<?> targetClass) throws Exception {
      return getInstances(targetClass,Integer.MAX_VALUE);
  }

  /**
   * 返回类的实例对象List
   * @param targetClass 需要查询实例的Class
   * @param limitNum 最多返回实例的数量
   * @return
   * @param <T>
   */
  public static <T> List<T> getInstanceList(Class<T> targetClass,int limitNum) throws Exception {
      List<T> result = new ArrayList<>();
      Object[] instances = getInstances(targetClass,limitNum);
      for(Object o : instances) {
          result.add((T)o);
      }
      return result;
  }

  /**
   * 获取类的所有实例对象List
   * @param targetClass 需要查询实例的Class
   * @return
   * @param <T>
   */
  public static <T> List<T> getInstanceList(Class<T> targetClass) throws Exception {
      return getInstanceList(targetClass,Integer.MAX_VALUE);
  }

  private final int i;

  public App(int i){
    this.i = i;
  }

  public static void main(String[] args){
    try {
      System.out.println(helloJNI());
    } catch(Exception e){
      System.out.println("Caught exception: " + e.getMessage());
      e.printStackTrace();
    }

    List<App> objs = new ArrayList<>();
    for(int i = 0; i < 10;i++) {
        objs.add(new App(i));
    }
    try{
      List<App> findApps = getInstanceList(App.class);
      findApps.sort(Comparator.comparing(o -> o.i));
      System.out.println("App所有创建实例:" + objs);
      System.out.println("App所有搜索实例:" + findApps);
      System.out.println("equals:" + objs.equals(findApps));
    }catch(Exception e){
      System.out.println("Caught exception: " + e.getMessage());
      e.printStackTrace();
    }
  }

  @Override
  public String toString(){
    return String.valueOf(this.i);
  }

  @Override
  public boolean equals(Object obj){
    return this.i == ((App) obj).i;
  }
}