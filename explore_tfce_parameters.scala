val N = 15 // subject count
val T = 30 // time point count

val r = new scala.util.Random(19717)

def generate_subject(): Array[Double] = {
  Array.fill(T)(r.nextGaussian().toFloat)
}

def apply_effect(orig_data: Array[Double], center: Int, size: Double, amplitude: Double) = {
  val data = orig_data.clone()
  val spread = math.floor(size*2).toInt
  (-spread to spread).foreach { d =>
    data(center+d) = 1f / math.sqrt(2 * math.Pi * size*size) * math.exp(-d*d/(2*size*size)) * amplitude
  }
  data
}

def tfce_above(data: Array[Double], result: Array[Double]) {
  val max = data.max(scala.math.Ordering.Double)
  if (max <= 0) return;
  val delta = max / 50
  (delta/2 to max by delta).foreach { t =>
    var stt = 0
    var end = 0
    (0 until data.length).foreach { i =>
      if (data(i) < t) {
        if (end > stt) {
          val cluster_size = end - stt
          (stt until end).foreach { i2 =>
            result(i2) += math.pow(cluster_size, 2.0/3.0) * t*t*delta
          }
        }
        stt = i
        end = i
      } else {
        if (end == stt) {
          stt = i
        }
        end = i + 1
      }
    }
    if (end > stt) {
      val cluster_size = end - stt
      (stt until end).foreach { i2 =>
        result(i2) += math.pow(cluster_size, 2.0/3.0) * t*t*delta
      }
    }
  }
}

def tfce_below(data: Array[Double], result: Array[Double]) {
  val min = data.min(scala.math.Ordering.Double)
  if (min >= 0) return;
  val delta = min / 50
  (delta/2 to min by delta).foreach { t =>
    var stt = 0
    var end = 0
    (0 until data.length).foreach { i =>
      if (data(i) > t) {
        if (end > stt) {
          val cluster_size = end - stt
          (stt until end).foreach { i2 =>
            result(i2) += math.pow(cluster_size, 2.0/3.0) * t*t*delta
          }
        }
        stt = i
        end = i
      } else {
        if (end == stt) {
          stt = i
        }
        end = i + 1
      }
    }
    if (end > stt) {
      val cluster_size = end - stt
      (stt until end).foreach { i2 =>
        result(i2) += math.pow(cluster_size, 2.0/3.0) * t*t*delta
      }
    }
  }
}

def tfce(data: Array[Double]): Array[Double] = {
  val result = new Array[Double](data.length)
  tfce_above(data, result)
  tfce_below(data, result)
  result
}

def ttest(
  condA: List[Array[Double]],
  condB: List[Array[Double]]
): Array[Double] = {
  (0 until T).map { t =>
    var sum = 0d
    var sum2 = 0d
    (0 until N).map { n =>
      val v = condA(n)(t) - condB(n)(t)
      sum += v
      sum2 += v*v
    }
    sum / math.sqrt((sum2*N - sum*sum)/(N-1))
  }.toArray
}

def permutation(
  condA: List[Array[Double]],
  condB: List[Array[Double]],
  iterations: Int
): Array[Boolean] = {
  val ts = (0 until iterations).map { i =>
    // printf("permutation iteration %d\n", i)
    val swaps = List.fill(N)(r.nextBoolean())
    val condAs = swaps.zipWithIndex.map { case (s,i) => if (s) condA(i) else condB(i) }
    val condBs = swaps.zipWithIndex.map { case (s,i) => if (s) condB(i) else condA(i) }
    tfce(ttest(condAs, condBs)).map(math.abs).max(scala.math.Ordering.Double)
  }
  val threshold = ts.sorted.apply(math.floor(iterations*0.95).toInt)
  printf("threshold = %.10f\n", threshold)
  tfce(ttest(condA, condB)).map(t => math.abs(t) > threshold)
}

def extract_result(output: Array[Boolean], effectCenter: Int, effectSize: Double): Boolean = {
  (math.ceil(-effectSize*2).toInt + effectCenter to math.floor(effectSize*2).toInt + effectCenter)
  .exists(i => output(i))
}

val condA = List.fill(N)(generate_subject())

val effectSize = 3
val effectCenter = T / 2 + math.round((r.nextDouble()-0.5)*(T - effectSize*4)).toInt
printf("c = %d\n", effectCenter)
val condB = List.fill(N)(apply_effect(generate_subject(), effectCenter, effectSize, 2))

println(extract_result(permutation(condA, condB, 100), effectCenter, effectSize))
