import { useCallback, useEffect, useRef, useState } from 'react';
import {
  EngineResponse,
  EngineSnapshot,
  JsBestMove,
  JsBoardState,
  JsMove,
} from '../types/damath';

interface PendingRequest<T> {
  resolve: (value: T) => void;
  reject: (reason?: unknown) => void;
}

export function useEngineWorker() {
  const workerRef = useRef<Worker | null>(null);
  const requestIdRef = useRef(0);
  const pendingRef = useRef(
    new Map<number, PendingRequest<unknown>>(),
  );

  const [isReady, setIsReady] = useState(false);
  const [isCalculating, setIsCalculating] = useState(false);
  const [boardState, setBoardState] = useState<JsBoardState | null>(null);
  const [legalMoves, setLegalMoves] = useState<JsMove[]>([]);
  const [engineError, setEngineError] = useState<string | null>(null);

  const applySnapshot = useCallback((snapshot: EngineSnapshot) => {
    setBoardState(snapshot.state);
    setLegalMoves(snapshot.legalMoves);
  }, []);

  useEffect(() => {
    const worker = new Worker(
      new URL('../workers/engine.worker.ts', import.meta.url),
      { type: 'module' },
    );

    workerRef.current = worker;

    const handleMessage = (event: MessageEvent<EngineResponse>) => {
      const message = event.data;

      if (message.type === 'READY') {
        applySnapshot(message.payload);
        setIsReady(true);
        return;
      }

      if (message.type === 'ERROR') {
        if (message.requestId !== undefined) {
          const pending = pendingRef.current.get(message.requestId);

          if (pending) {
            pendingRef.current.delete(message.requestId);
            pending.reject(message.payload);
          }
        }

        setEngineError(message.payload);
        return;
      }

      const pending = pendingRef.current.get(message.requestId);

      if (!pending) {
        return;
      }

      pendingRef.current.delete(message.requestId);

      if (message.type === 'RESULT') {
        applySnapshot(message.payload);
        pending.resolve(message.payload);
      } else if (message.type === 'BEST_MOVE_RESULT') {
        pending.resolve(message.payload);
      }
    };

    worker.addEventListener('message', handleMessage);
    worker.postMessage({ type: 'INIT' });

    return () => {
      worker.removeEventListener('message', handleMessage);
      worker.terminate();

      for (const pending of pendingRef.current.values()) {
        pending.reject(new Error('Engine worker was terminated.'));
      }

      pendingRef.current.clear();
      workerRef.current = null;
    };
  }, [applySnapshot]);

  const makeRequest = useCallback(<T,>(
    message: Record<string, unknown>,
  ): Promise<T> => {
    const worker = workerRef.current;

    if (!worker) {
      return Promise.reject(new Error('Engine worker is unavailable.'));
    }

    const requestId = ++requestIdRef.current;

    return new Promise<T>((resolve, reject) => {
      pendingRef.current.set(requestId, {
        resolve: resolve as (value: unknown) => void,
        reject,
      });

      worker.postMessage({
        ...message,
        requestId,
      });
    });
  }, []);

  const makeMove = useCallback(
    async (move: JsMove): Promise<EngineSnapshot> => {
      setEngineError(null);

      return makeRequest<EngineSnapshot>({
        type: 'MAKE_MOVE',
        payload: move,
      });
    },
    [makeRequest],
  );

  const findBestMove = useCallback(
    async (depth: number, timeLimitMs: number): Promise<JsBestMove | null> => {
      setEngineError(null);
      setIsCalculating(true);

      try {
        return await makeRequest<JsBestMove | null>({
          type: 'FIND_BEST_MOVE',
          payload: { depth, timeLimitMs },
        });
      } finally {
        setIsCalculating(false);
      }
    },
    [makeRequest],
  );

  const makeBestMove = useCallback(
    async (depth: number, timeLimitMs: number): Promise<EngineSnapshot> => {
      setEngineError(null);
      setIsCalculating(true);

      try {
        return await makeRequest<EngineSnapshot>({
          type: 'MAKE_BEST_MOVE',
          payload: { depth, timeLimitMs },
        });
      } finally {
        setIsCalculating(false);
      }
    },
    [makeRequest],
  );

  const reset = useCallback(async (): Promise<EngineSnapshot> => {
    setEngineError(null);

    return makeRequest<EngineSnapshot>({
      type: 'RESET',
    });
  }, [makeRequest]);

  return {
    isReady,
    isCalculating,
    boardState,
    legalMoves,
    engineError,
    makeMove,
    findBestMove,
    makeBestMove,
    reset,
  };
}
