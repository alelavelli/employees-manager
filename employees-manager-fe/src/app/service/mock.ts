import { HttpEventType, HttpResponse } from '@angular/common/http';
import { Observable } from 'rxjs';

const MOCK_TIME = 1000;

export const buildMocked = <T>(response?: T): Observable<T> => {
  return new Observable<T>((subscriber) => {
    setTimeout(() => {
      subscriber.next(response);
      subscriber.complete();
    }, MOCK_TIME);
  });
};
export const buildMockedError = <T>(response?: T): Observable<T> => {
  return new Observable<T>((subscriber) => {
    setTimeout(() => {
      subscriber.error(response);
      subscriber.complete();
    }, MOCK_TIME);
  });
};
export const buildMockedProgress = () =>
  new Observable((subscriber) => {
    let progress = 0;
    const intervalId = setInterval(() => {
      if (progress > 100) {
        clearInterval(intervalId);
        subscriber.next(new HttpResponse());
        subscriber.complete();
      } else {
        subscriber.next({
          type: HttpEventType.UploadProgress,
          loaded: progress,
          total: 100,
        });
        progress += 25;
      }
    }, MOCK_TIME);
  });
