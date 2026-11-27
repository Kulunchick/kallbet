import numpy as np
import pytest

torch = pytest.importorskip("torch")

from kbml import pickem_check, swiss_gpu  # noqa: E402
from kbml.pickem_data import Stage  # noqa: E402

MAJOR = {(w, l): 3 if 2 in (w, l) else 1 for w in range(3) for l in range(3)}


def stage(published, opening, seed=None):
    return Stage(1, "test", 0, list(map(str, range(16))), opening, MAJOR, None, published, True,
                 list(range(16)), seed)


@pytest.mark.parametrize("dev", ["cpu", "cuda"])
@pytest.mark.parametrize("case", range(4))
def test_same_uniforms_give_the_numpy_outcomes(dev, case):
    if dev == "cuda" and not torch.cuda.is_available():
        pytest.skip("no CUDA device")
    gen = np.random.default_rng(case)
    values = gen.uniform(0.05, 0.95, 240)
    if case % 2:
        s = stage(False, [tuple(p) for p in gen.permutation(16).reshape(8, 2)], gen.permutation(16))
    else:
        s = stage(True, [(i, i + 8) for i in range(8)])
    runs = 3000
    expected = pickem_check.simulate(s, values, runs, np.random.default_rng(7))
    rng = np.random.default_rng(7)
    device = torch.device(dev)
    got = swiss_gpu.simulate(s, values, runs, device,
                             lambda n: torch.as_tensor(rng.random(n), device=device)).cpu().numpy()
    np.testing.assert_array_equal(got, expected)
    assert (np.sort(got, axis=1) == np.repeat(np.arange(4), [2, 2, 6, 6])).all()
