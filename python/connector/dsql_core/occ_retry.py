# Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0

from __future__ import annotations

import asyncio
import logging
import random
import time
from collections.abc import Awaitable, Callable
from dataclasses import dataclass
from typing import TypeVar

T = TypeVar("T")
logger = logging.getLogger(__name__)


@dataclass(frozen=True)
class OCCRetryConfig:
    """Retry configuration for OCC conflict handling.

    Attributes:
        max_retries: Additional attempts after the initial try (0-100).
        base_delay_ms: Initial backoff delay in milliseconds (must be > 0).
        max_delay_ms: Backoff delay cap in milliseconds (must be >= base_delay_ms).
        jitter_factor: Fraction of delay added as random jitter (0.0-1.0).
    """

    max_retries: int = 3
    base_delay_ms: int = 1
    max_delay_ms: int = 100
    jitter_factor: float = 0.25

    def __post_init__(self) -> None:
        if not isinstance(self.max_retries, int):
            raise ValueError("max_retries must be an integer")
        if self.max_retries < 0:
            raise ValueError("max_retries must be >= 0")
        if self.max_retries > 100:
            raise ValueError("max_retries must not exceed 100")
        if self.base_delay_ms <= 0:
            raise ValueError("base_delay_ms must be greater than 0")
        if self.max_delay_ms < self.base_delay_ms:
            raise ValueError("max_delay_ms must be >= base_delay_ms")
        if self.jitter_factor < 0.0 or self.jitter_factor > 1.0:
            raise ValueError("jitter_factor must be between 0.0 and 1.0")


def resolve_retry_config(
    pool_config: OCCRetryConfig | bool | None,
    per_call: OCCRetryConfig | bool | None,
) -> OCCRetryConfig | None:
    """Resolve effective retry config from pool and per-call settings."""
    if isinstance(per_call, OCCRetryConfig):
        return per_call
    if per_call is False:
        return None
    if per_call is True:
        if isinstance(pool_config, OCCRetryConfig):
            return pool_config
        return OCCRetryConfig()
    if isinstance(pool_config, OCCRetryConfig):
        return pool_config
    if pool_config is True:
        return OCCRetryConfig()
    return None


def is_occ_error(error: Exception) -> bool:
    """Detect OCC conflict by checking sqlstate or pgcode for OC000, OC001, or 40001."""
    code = getattr(error, "sqlstate", None)
    if code is None:
        code = getattr(error, "pgcode", None)
    if code is None:
        return False
    return code in ("OC000", "OC001", "40001")


def _calculate_backoff(config: OCCRetryConfig, attempt: int) -> float:
    """Compute backoff delay in ms: min(base * 2^(attempt-1), max) + jitter."""
    exponent = min(attempt - 1, 31)
    delay = min(config.base_delay_ms * (2.0**exponent), config.max_delay_ms)
    jitter = delay * random.random() * config.jitter_factor
    return delay + jitter


def _retry_sync(execute: Callable[[], T], config: OCCRetryConfig) -> T:
    """Sync retry loop. Retries the thunk on OCC errors with backoff."""
    for attempt in range(config.max_retries + 1):
        try:
            return execute()
        except Exception as e:
            if not is_occ_error(e) or attempt == config.max_retries:
                raise
            delay = _calculate_backoff(config, attempt + 1)
            logger.debug(
                "OCC conflict on attempt %d/%d, retrying in %.1fms",
                attempt + 1,
                config.max_retries + 1,
                delay,
            )
            time.sleep(delay / 1000.0)
    raise AssertionError("unreachable")


async def _retry_async(execute: Callable[[], Awaitable[T]], config: OCCRetryConfig) -> T:
    """Async retry loop. Retries the thunk on OCC errors with backoff."""
    for attempt in range(config.max_retries + 1):
        try:
            return await execute()
        except Exception as e:
            if not is_occ_error(e) or attempt == config.max_retries:
                raise
            delay = _calculate_backoff(config, attempt + 1)
            logger.debug(
                "OCC conflict on attempt %d/%d, retrying in %.1fms",
                attempt + 1,
                config.max_retries + 1,
                delay,
            )
            await asyncio.sleep(delay / 1000.0)
    raise AssertionError("unreachable")
