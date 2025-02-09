import { bootstrapApplication } from '@angular/platform-browser';
import { appConfig } from './app/app.config';
import { environment } from './environments/environment';
import { AppComponent } from './app/components/app/app.component';

console.log(
  '%cemployees-manager-fe env:%c' + environment.envName + '',
  'background: #1c283b; color: #FFFFFF; font-size:30px',
  'background: #1c283b; color: #8ec047; font-size:30px'
);

bootstrapApplication(AppComponent, appConfig).catch((err) =>
  console.error(err)
);
