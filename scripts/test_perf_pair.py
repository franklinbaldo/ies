"""Regressões do contrato do comparador; somente dados sintéticos."""
import math
import unittest
from perf_pair import canonical, percentile, validate, summarize


class PairTests(unittest.TestCase):
    def test_nearest_rank(self):
        self.assertEqual(percentile(list(range(1, 26)), .95), 24)

    def test_duplicate_shared_between_tables(self):
        self.assertEqual(canonical('U', [('01', 'N'), ('01', 'N')]),
                         {'unidade': 'U', 'processos': [('01', 'N')]})

    def test_same_number_different_id_is_error(self):
        with self.assertRaises(ValueError):
            canonical('U', [('01', 'N'), ('02', 'N')])

    def test_budget_and_numeric_rejections(self):
        good = dict(total_ns=100, cpu_ns=80, mock_wait_ns=0,
                    requests=1, response_bytes=10)
        validate(good, 10)
        for key, value in [('requests', 2), ('response_bytes', 9),
                           ('cpu_ns', -1), ('total_ns', math.nan),
                           ('mock_wait_ns', 101)]:
            with self.subTest(key=key), self.assertRaises(ValueError):
                validate({**good, key: value}, 10)

    def test_changed_identity_rejected_before_reporting(self):
        expected = canonical('U', [('1', 'N')])
        changed = canonical('U', [('2', 'N')])
        with self.assertRaises(ValueError):
            summarize(dict(samples=[{}] * 5, canonical=changed), expected, 10, 5)

    def test_percentiles_subtract_wait_per_sample(self):
        value = canonical('U', [('1', 'N')])
        result = dict(canonical=value, rss_peak_kib=100, samples=[
            dict(total_ns=100+i, cpu_ns=30, mock_wait_ns=i,
                 requests=1, response_bytes=10) for i in range(5)])
        report = summarize(result, value, 10, 5)
        self.assertEqual(report['local_ns'], dict(p50=100, p95=100))
        self.assertEqual(report['network'], 'NOT_RUN')


if __name__ == '__main__':
    unittest.main()
