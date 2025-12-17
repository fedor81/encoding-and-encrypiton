import sys
import numpy as np
import matplotlib.pyplot as plt

if len(sys.argv) != 2:
    print("Использование: python3 plot.py <файл_с_данными>")
    sys.exit(1)

filename = sys.argv[1]

# Загрузка данных
with open(filename, 'r') as f:
    data = [float(line.strip()) for line in f if line.strip()]

data = np.array(data)
N = len(data)

# 1. Среднее и дисперсия (дублируем для проверки)
mean = np.mean(data)
var = np.var(data)  # population variance (ddof=0)
print(f"[Python] Среднее: {mean:.6f}")
print(f"[Python] Дисперсия: {var:.6f}")

# 2. Гистограмма
plt.figure(figsize=(12, 5))

plt.subplot(1, 2, 1)
plt.hist(data, bins=50, edgecolor='black', alpha=0.7)
plt.title('Гистограмма')
plt.xlabel('Значение')
plt.ylabel('Частота')

# 3. Диаграмма рассеяния: (x[i], x[i+1])
plt.subplot(1, 2, 2)
plt.scatter(data[:-1], data[1:], s=0.1, alpha=0.6)
plt.title('Последовательные пары')
plt.xlabel('$x_i$')
plt.ylabel('$x_{i+1}$')

plt.tight_layout()
plt.savefig('prng_quality.png', dpi=150)
plt.show()