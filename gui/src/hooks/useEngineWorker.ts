import { useEffect, useRef, useState, useCallback } from 'react';
import { JsBestMove, JsMove } from '../types/damath';

export function useEngineWorker() {
  const workerRef = useRef<Worker | null>(null);
  const [isReady, setIsReady] = useState(false);
  const [isCalculating, setIsCalculating] = useState(false);

  useEffect(() => {
    const worker = new Worker(
      new URL('../workers/engine.worker.ts', import.meta.url),
      { type: 'module' }
    );

    workerRef.current = worker;

    const handleMessage = (e: MessageEvent) => {
      if (e.data.type === 'READY') {
        setIsReady(true);
      }
    };

    worker.addEventListener('message', handleMessage);
    worker.postMessage({ type: 'INIT' });

    return () => {
      worker.removeEventListener('message', handleMessage);
      worker.terminate();
    };
  }, []);

  const findBestMove = useCallback(
    (depth: number, timeLimitMs: number, movesHistory: JsMove[] = []): Promise<JsBestMove | null> => {
      return new Promise((resolve) => {
        if (!workerRef.current) {
          resolve(null);
          return;
        }

        setIsCalculating(true);

        const handleResult = (e: MessageEvent) => {
          if (e.data.type === 'BEST_MOVE_RESULT') {
            setIsCalculating(false);
            workerRef.current?.removeEventListener('message', handleResult);
            resolve(e.data.payload);
          }
        };

        workerRef.current.addEventListener('message', handleResult);
        workerRef.current.postMessage({
          type: 'FIND_BEST_MOVE',
          payload: { depth, timeLimitMs, movesHistory },
        });
      });
    },
    []
  );

  return { isReady, isCalculating, findBestMove };
}