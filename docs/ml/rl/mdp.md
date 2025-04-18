# 马尔可夫决策过程

## 环境和智能代理的数学表示

MDP(Markov Decision Process) 通过数学式来表示智能代理、环境以及二者之间的互动。要做到这一点，需要用数学式来表达`状态迁移`，`奖励`，`策略`这3个要素。

### 状态迁移

对于随机性状态迁移：假设智能代理现在处于状态 $s$ 并执行了行动 $a$ ，那么迁移到下一个状态 $s'$ 的概率可以用如下方式表示。
$$
p(s'| s, a)
$$
$|$ 的右侧是表示"条件"的概率变量。像这样的概率叫作状态迁移概率（state transition probability）。

$p(s' | s, a)$决定了下一个状态 $s'$ 只取决于当前状态 $s$ 和行动 $a$。

换句话说，状态迁移不需要过去的信息——此前处于什么状态以及执行了哪些行动。这个特性被称为马尔可夫性（Markov property）。

### 奖励

假设奖励的发放是“确定性”的。当智能代理在处于状态 $s$，执行了行动 $a$ ，下一个状态是 $s'$时，得到的奖励由函数 $r(s, a, s')$ 定义。这个函数称为奖励函数（reward function）。

智能代理与环境之间做出的一系列（或一回合）的交互后得到的状态、动作、奖励所构成的序列，称为运动轨迹（episodes）。例如智能代理与环境交互 $T$ 次：
$$
\tau = (s_0, a_0, r_0, s_1, a_1, r_1, \dots, s_{T_1}, a_{T_1}, r_{T-1})
$$

### 策略

智能代理的行动是由随机性策略决定的，数学式如下所示。
$$
π(a|s)
$$
其表示在状态 $s$ 下采取行动 $a$ 的概率。

## MDP 的目标

收益（return）被表示为智能代理获得的奖励之和。智能代理的目标是使收益最大化。
$$
\begin{aligned}
G_t &= R_t + {\gamma}R_{t + 1} + {\gamma}^2 R_{t + 2} + \cdots \\
&= R_t + {\gamma} ( R_{t + 1} + {\gamma} R_{t + 2} + \cdots ) \\
&= R_t + {\gamma} G_{t + 1}
\end{aligned}
$$

其中，$\gamma$ 成为折现率（discount rate），这使得近期的奖励显得更加重要。

为了处理智能代理的随机行动，需要使用期望值或“收益的期望值”作为衡量标准。收益的期望值的数学式如下所示。
$$
v_{\pi}(s) = \mathbb{E}_{\pi} [G_t | S_t = s]
$$
其中，我们指定的条件是状态 $S_t$ 为 $s$、智能代理的策略为 $\pi$（时刻 t 是任意值）。$v_{\pi}(s)$ 被称为状态价值函数（state-value function）。

MDP 的目标就是找到最优策略，即找到使收益最大化的策略。

> 在 MDP 中，至少存在一个最优策略。最优策略是确定行策略。
>
> 证明过程可参考：[Algorithms for Reinforcement Learning](https://sites.ualberta.ca/~szepesva/papers/RLAlgsInMDPs.pdf)

最优策略的状态价值函数叫做最优状态价值函数（optimal state-value function）。
