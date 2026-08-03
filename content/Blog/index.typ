#import "../index.typ": template, tufted
#show: template.with(
  title: "博客",
  description: "窝的博客",
)

= 博客

== 2026

#tufted.blog-entry(
  date: datetime(year: 2026, month: 4, day: 25),
  path: "2026-04-25-ros2-dds-qos/",
  title: "ROS2、DDS 与 QoS",
)

== 2025

#tufted.blog-entry(
  date: datetime(year: 2025, month: 10, day: 30),
  path: "2025-10-30-normal-distribution/",
  title: "The Normal Distribution: A Fundamental Concept in Statistics",
)

== 2024

#tufted.blog-entry(
  date: datetime(year: 2024, month: 10, day: 4),
  path: "2024-10-04-iterators-generators/",
  title: "Iterators vs Generators in Python",
)
