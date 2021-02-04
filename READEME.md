# 介绍
这是基于libpnet的网络数据包统计库，
为Mac小应用[网络监控](https://github.com/QaQAdrian/monitor)提供后端支持。学习为主，有疑问可以讨论。

## Target
目前有两个target
- bin： 作为rust调试用
- staticlib：编译成静态库，兼容C ABI支持Swift项目调用

## 关于线程
- 每个可用、有ip的网卡单独使用一个线程进行收集数据，出现异常则结束此线程。(n网卡n线程)
- 一个线程将所有网卡的数据汇总（mpsc channel）
- 主线程（静态库为调用时的线程，需要调用方主动退出，懒得做。。）通过Mutex和汇总数据的线程通信。