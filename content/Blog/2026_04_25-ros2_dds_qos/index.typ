#import "../index.typ": template, tufted
#import "@preview/theorion:0.4.1": *
#import "@preview/lilaq:0.5.0" as lq
#show: template.with(
  title: "ROS2、DDS 与 QoS",
  description: "我在使用 ROS2 的时候遇到的关于 DDS 的问题与解决过程。",
  date: datetime(year: 2026, month: 04, day: 25),
  lang: "zh",
)

= ROS2、DDS 与 QoS

#quote-box[
  我参加的比赛要求在树莓派上使用视觉算法。为了更好的调试代码，一个可以实时显示图像的可视化界面必不可少。但是我并不想用虚拟机，恰好我的 Windows 电脑支持 WSLg，又恰好比赛用到的 ROS2 支持分布式通信（理论上我可以在 WSL2 上查看同一局域网内的树莓派上的 ROS2 话题等等），于是用WSL2查看树莓派的视觉图像的想法油然而生。然后我就发现用 WSL2 查看图像卡的不行，即使是 /image/compressed 下的压缩图像也有好几秒钟的延迟。询问 AI 后得知我应该换用 Cyclone DDS，那么，为什么呢？
]

== DDS

DDS（Data Distribution Service）是一个由对象管理组织（OMG）制定的分布式实时通信中间件技术规范，基于发布/订阅模型并以数据为中心，通过 QoS 服务质量策略保障数据传输的实时性、可靠性与安全性。DDS 广泛应用于各类场景，包括空中交通管制、喷气发动机测试、铁路控制、医疗系统、海军指挥控制、智能温室等等。

== ROS2中的DDS

它在航空航天和国防领域早已成熟，而如今已不再局限于这些领域。现代机器人系统既需要高吞吐量、低延迟控制系统，又要避免因中间引入消息代理而产生单点故障。DDS 已然是 ROS2 中使用最广泛、默认的中间件选择，用于在组件之间传输指令、传感器数据，甚至视频和点云数据。不同厂商实现了不同的DDS具体实现，ROS2 通过一个叫 RMW(ROS Middleware) 的抽象层来灵活切换它们。

而我用到的 ROS2 Humble 默认使用 Fast DDS。它 与Cyclone DDS 有什么区别呢？D老师#footnote[指 DeepSeek]帮我总结了以下三点：

- *跨网络高吞吐与低延迟*：Fast DDS 的优势在单机进程间通信，对跨网络、高吞吐场景的优化不足。尤其是在 WSL2 的虚拟网络环境下，图像、点云这类大数据流容易形成拥塞，延迟急剧上升。Cyclone DDS 则天生为跨网络、高吞吐场景设计，吞吐量更稳定，端到端延迟显著更低。
- *WSL2 兼容性*：Fast DDS 在 WSL2 中经常出现节点发现失败、延迟剧烈抖动和 CPU 占用过高等问题。Cyclone DDS 则提供了简洁的 XML 配置文件，可以显式指定 WSL2 的虚拟网卡，直接绕过了大部分网络兼容性陷阱。#footnote[但我并没有用到这一步。]
- *资源占用与设计哲学*：两者都适用于资源受限的嵌入式环境，但 Cyclone DDS 在设计上更追求跨平台和跨网络的鲁棒性，默认配置在分布式场景中表现更友好，也是官方文档和社区在解决类似问题时的高频推荐。

== QoS

DDS 最强大的特性之一就是 QoS（Quality of Service），它允许你精细控制数据的传输行为。我遭遇到的几秒延迟，除了中间件本身的因素，另一个重要推手就是不合适的 QoS 策略。

ROS2 中与我们的卡顿问题最相关的 QoS 策略有这几项：

- *可靠性（Reliability）*：分为 `RELIABLE` 和 `BEST_EFFORT`。`RELIABLE` 保证数据一定送达，丢包后会要求重传。在 WSL2 这类抖动较大的网络中，一个丢包就可能让后续所有数据包排队等待重传，造成“几秒延迟”的感觉。对于实时视觉图像，我们通常关心的是“最新一帧是什么”，而不是“每一帧都必须收到”。切换到 `BEST_EFFORT` 后，中间件会尽力发送，但不会因丢包而阻塞后续数据，延迟立刻大幅下降。
- *历史记录（History）与深度（Depth）*：`KEEP_LAST` 配合一个较小的队列深度（例如 `1`），让发布者只保留最新的一帧，订阅者拿到的永远是最新数据。这可以避免老旧帧堆积，进一步降低处理延迟。
- *持续性（Durability）*：一般设置为 `VOLATILE`（不保存历史），避免刚启动时收到大量老数据，尤其适合高频传感器流。

简单来说，最适合实时图像流的组合就是：*Best Effort + Keep Last(深度1) + Volatile*。当你在订阅端代码中将 QoS 配置成这样，配合 Cyclone DDS，即使跨 WSL2 查看树莓派上的图像，也能获得近乎实时的流畅体验。

== 具体实践

```bash
# 1. 安装 Cyclone DDS 的 ROS2 中间件
sudo apt install ros-humble-rmw-cyclonedds-cpp

# 2. 告诉 ROS2 换用 Cyclone DDS
export RMW_IMPLEMENTATION=rmw_cyclonedds_cpp
```
