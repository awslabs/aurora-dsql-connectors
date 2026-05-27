# Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0

from unittest.mock import patch

import pytest

from dsql_core.occ_retry import (
    OCCRetryConfig,
    _calculate_backoff,
    _retry_async,
    _retry_sync,
    is_occ_error,
    resolve_retry_config,
)


@pytest.mark.unit
class TestOCCRetryConfig:
    def test_defaults(self):
        config = OCCRetryConfig()
        assert config.max_retries == 3
        assert config.base_delay_ms == 1
        assert config.max_delay_ms == 100
        assert config.jitter_factor == 0.25

    def test_custom_values(self):
        config = OCCRetryConfig(max_retries=5, base_delay_ms=10, max_delay_ms=50, jitter_factor=0.5)
        assert config.max_retries == 5
        assert config.base_delay_ms == 10
        assert config.max_delay_ms == 50
        assert config.jitter_factor == 0.5

    def test_rejects_negative_max_retries(self):
        with pytest.raises(ValueError, match="max_retries must be >= 0"):
            OCCRetryConfig(max_retries=-1)

    def test_rejects_max_retries_over_100(self):
        with pytest.raises(ValueError, match="max_retries must not exceed 100"):
            OCCRetryConfig(max_retries=101)

    def test_rejects_zero_base_delay(self):
        with pytest.raises(ValueError, match="base_delay_ms must be greater than 0"):
            OCCRetryConfig(base_delay_ms=0)

    def test_rejects_negative_base_delay(self):
        with pytest.raises(ValueError, match="base_delay_ms must be greater than 0"):
            OCCRetryConfig(base_delay_ms=-1)

    def test_rejects_max_delay_less_than_base(self):
        with pytest.raises(ValueError, match="max_delay_ms must be >= base_delay_ms"):
            OCCRetryConfig(base_delay_ms=50, max_delay_ms=10)

    def test_rejects_negative_jitter(self):
        with pytest.raises(ValueError, match="jitter_factor must be between 0.0 and 1.0"):
            OCCRetryConfig(jitter_factor=-0.1)

    def test_rejects_jitter_over_one(self):
        with pytest.raises(ValueError, match="jitter_factor must be between 0.0 and 1.0"):
            OCCRetryConfig(jitter_factor=1.5)

    def test_accepts_zero_retries(self):
        config = OCCRetryConfig(max_retries=0)
        assert config.max_retries == 0

    def test_accepts_boundary_values(self):
        config = OCCRetryConfig(
            max_retries=100, base_delay_ms=1, max_delay_ms=100, jitter_factor=1.0
        )
        assert config.max_retries == 100
        assert config.jitter_factor == 1.0

    def test_frozen(self):
        config = OCCRetryConfig()
        with pytest.raises(Exception):
            config.max_retries = 5  # type: ignore[misc]


@pytest.mark.unit
class TestResolveRetryConfig:
    def test_both_none(self):
        assert resolve_retry_config(None, None) is None

    def test_pool_config_none_per_call_none(self):
        assert resolve_retry_config(None, None) is None

    def test_pool_true_per_call_none(self):
        result = resolve_retry_config(True, None)
        assert isinstance(result, OCCRetryConfig)
        assert result.max_retries == 3

    def test_pool_config_per_call_none(self):
        cfg = OCCRetryConfig(max_retries=5)
        result = resolve_retry_config(cfg, None)
        assert result is cfg

    def test_pool_false_per_call_none(self):
        assert resolve_retry_config(False, None) is None

    def test_pool_any_per_call_false(self):
        cfg = OCCRetryConfig(max_retries=5)
        assert resolve_retry_config(cfg, False) is None
        assert resolve_retry_config(True, False) is None
        assert resolve_retry_config(None, False) is None

    def test_pool_any_per_call_true(self):
        cfg = OCCRetryConfig(max_retries=5)
        result = resolve_retry_config(cfg, True)
        assert result is cfg

    def test_pool_true_per_call_true(self):
        result = resolve_retry_config(True, True)
        assert isinstance(result, OCCRetryConfig)

    def test_pool_none_per_call_true(self):
        result = resolve_retry_config(None, True)
        assert isinstance(result, OCCRetryConfig)
        assert result.max_retries == 3

    def test_per_call_config_replaces_pool(self):
        pool_cfg = OCCRetryConfig(max_retries=5)
        call_cfg = OCCRetryConfig(max_retries=10)
        result = resolve_retry_config(pool_cfg, call_cfg)
        assert result is call_cfg


@pytest.mark.unit
class TestIsOCCError:
    def test_oc000_via_sqlstate(self):
        err = Exception()
        err.sqlstate = "OC000"  # type: ignore[attr-defined]
        assert is_occ_error(err) is True

    def test_oc001_via_sqlstate(self):
        err = Exception()
        err.sqlstate = "OC001"  # type: ignore[attr-defined]
        assert is_occ_error(err) is True

    def test_40001_via_sqlstate(self):
        err = Exception()
        err.sqlstate = "40001"  # type: ignore[attr-defined]
        assert is_occ_error(err) is True

    def test_oc000_via_pgcode(self):
        err = Exception()
        err.pgcode = "OC000"  # type: ignore[attr-defined]
        assert is_occ_error(err) is True

    def test_oc001_via_pgcode(self):
        err = Exception()
        err.pgcode = "OC001"  # type: ignore[attr-defined]
        assert is_occ_error(err) is True

    def test_40001_via_pgcode(self):
        err = Exception()
        err.pgcode = "40001"  # type: ignore[attr-defined]
        assert is_occ_error(err) is True

    def test_non_occ_code(self):
        err = Exception()
        err.sqlstate = "23505"  # type: ignore[attr-defined]
        assert is_occ_error(err) is False

    def test_no_code_attribute(self):
        err = Exception("generic error")
        assert is_occ_error(err) is False

    def test_none_code(self):
        err = Exception()
        err.sqlstate = None  # type: ignore[attr-defined]
        assert is_occ_error(err) is False

    def test_sqlstate_takes_precedence_over_pgcode(self):
        err = Exception()
        err.sqlstate = "OC000"  # type: ignore[attr-defined]
        err.pgcode = "23505"  # type: ignore[attr-defined]
        assert is_occ_error(err) is True


@pytest.mark.unit
class TestCalculateBackoff:
    def test_first_attempt(self):
        config = OCCRetryConfig(base_delay_ms=1, max_delay_ms=100, jitter_factor=0.0)
        delay = _calculate_backoff(config, 1)
        assert delay == 1.0

    def test_second_attempt(self):
        config = OCCRetryConfig(base_delay_ms=1, max_delay_ms=100, jitter_factor=0.0)
        delay = _calculate_backoff(config, 2)
        assert delay == 2.0

    def test_third_attempt(self):
        config = OCCRetryConfig(base_delay_ms=1, max_delay_ms=100, jitter_factor=0.0)
        delay = _calculate_backoff(config, 3)
        assert delay == 4.0

    def test_caps_at_max_delay(self):
        config = OCCRetryConfig(base_delay_ms=1, max_delay_ms=100, jitter_factor=0.0)
        delay = _calculate_backoff(config, 10)
        assert delay == 100.0

    def test_jitter_adds_randomness(self):
        config = OCCRetryConfig(base_delay_ms=10, max_delay_ms=100, jitter_factor=0.5)
        delay = _calculate_backoff(config, 1)
        assert delay >= 10.0
        assert delay <= 15.0  # 10 + 10*0.5

    def test_exponent_capped_at_31(self):
        config = OCCRetryConfig(base_delay_ms=1, max_delay_ms=10**12, jitter_factor=0.0)
        delay = _calculate_backoff(config, 50)
        assert delay == 2**31


@pytest.mark.unit
class TestRetrySyncLoop:
    def test_succeeds_first_try(self):
        result = _retry_sync(lambda: "ok", OCCRetryConfig())
        assert result == "ok"

    @patch("dsql_core.occ_retry.time.sleep")
    def test_retries_on_occ_then_succeeds(self, mock_sleep):
        attempts = []

        def execute():
            attempts.append(1)
            if len(attempts) < 3:
                err = Exception()
                err.sqlstate = "OC000"  # type: ignore[attr-defined]
                raise err
            return "recovered"

        result = _retry_sync(execute, OCCRetryConfig())
        assert result == "recovered"
        assert len(attempts) == 3
        assert mock_sleep.call_count == 2

    @patch("dsql_core.occ_retry.time.sleep")
    def test_exhausts_retries(self, mock_sleep):
        attempts = []

        def execute():
            attempts.append(1)
            err = Exception()
            err.sqlstate = "OC000"  # type: ignore[attr-defined]
            raise err

        with pytest.raises(Exception) as exc_info:
            _retry_sync(execute, OCCRetryConfig(max_retries=3))

        assert hasattr(exc_info.value, "sqlstate")
        assert len(attempts) == 4  # 1 initial + 3 retries

    def test_non_occ_error_not_retried(self):
        attempts = []

        def execute():
            attempts.append(1)
            raise ValueError("not an OCC error")

        with pytest.raises(ValueError, match="not an OCC error"):
            _retry_sync(execute, OCCRetryConfig())

        assert len(attempts) == 1

    def test_zero_retries_executes_once(self):
        attempts = []

        def execute():
            attempts.append(1)
            err = Exception()
            err.sqlstate = "OC000"  # type: ignore[attr-defined]
            raise err

        with pytest.raises(Exception):
            _retry_sync(execute, OCCRetryConfig(max_retries=0))

        assert len(attempts) == 1


@pytest.mark.unit
class TestRetryAsyncLoop:
    @pytest.mark.asyncio
    async def test_succeeds_first_try(self):
        async def execute():
            return "ok"

        result = await _retry_async(execute, OCCRetryConfig())
        assert result == "ok"

    @pytest.mark.asyncio
    @patch("dsql_core.occ_retry.asyncio.sleep")
    async def test_retries_on_occ_then_succeeds(self, mock_sleep):
        mock_sleep.return_value = None
        attempts = []

        async def execute():
            attempts.append(1)
            if len(attempts) < 3:
                err = Exception()
                err.sqlstate = "OC000"  # type: ignore[attr-defined]
                raise err
            return "recovered"

        result = await _retry_async(execute, OCCRetryConfig())
        assert result == "recovered"
        assert len(attempts) == 3

    @pytest.mark.asyncio
    @patch("dsql_core.occ_retry.asyncio.sleep")
    async def test_exhausts_retries(self, mock_sleep):
        mock_sleep.return_value = None
        attempts = []

        async def execute():
            attempts.append(1)
            err = Exception()
            err.sqlstate = "OC000"  # type: ignore[attr-defined]
            raise err

        with pytest.raises(Exception) as exc_info:
            await _retry_async(execute, OCCRetryConfig(max_retries=3))

        assert hasattr(exc_info.value, "sqlstate")
        assert len(attempts) == 4

    @pytest.mark.asyncio
    async def test_non_occ_error_not_retried(self):
        attempts = []

        async def execute():
            attempts.append(1)
            raise ValueError("not an OCC error")

        with pytest.raises(ValueError, match="not an OCC error"):
            await _retry_async(execute, OCCRetryConfig())

        assert len(attempts) == 1

    @pytest.mark.asyncio
    async def test_zero_retries_executes_once(self):
        attempts = []

        async def execute():
            attempts.append(1)
            err = Exception()
            err.sqlstate = "OC000"  # type: ignore[attr-defined]
            raise err

        with pytest.raises(Exception):
            await _retry_async(execute, OCCRetryConfig(max_retries=0))

        assert len(attempts) == 1
