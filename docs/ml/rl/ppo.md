# PPO

主要思想是，在更新之后，新策略应该与旧策略相差不远。为此，ppo 使用裁剪来避免过大的更新。