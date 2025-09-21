import { Moment } from 'moment';
import { CompanyRole, NotificationType, TimesheetDayWorkType } from './enums';

export interface LoginResponse {
  token: string;
  tokenType: string;
}

export interface UserData {
  id: string;
  username: string;
  email: string;
  name: string;
  surname: string;
  platformAdmin: boolean;
  active: boolean;
}

export interface AdminPanelOverview {
  totalUsers: number;
  totalAdmins: number;
  totalActiveUsers: number;
  totalInactiveUsers: number;
  totalCompanies: number;
}

export interface AdminPanelUserInfo {
  id: string;
  username: string;
  email: string;
  name: string;
  surname: string;
  platformAdmin: boolean;
  active: boolean;
  totalCompanies: number;
}

export interface CreateUserParameters {
  username: string;
  password: string;
  name: string;
  surname: string;
  email: string;
}

export interface AppNotification {
  id: string;
  notificationType: NotificationType;
  message: string;
}

export interface CompanyInfo {
  id: string;
  name: string;
  active: boolean;
  totalUsers: number;
  role: CompanyRole;
}

export interface UserInCompanyInfo {
  userId: string;
  userUsername: string;
  userName: string;
  userSurname: string;
  jobTitle: string;
  companyId: string;
  role: CompanyRole;
  job_title: string;
  managementTeam: boolean;
}

export interface CreateCompanyParameters {
  name: string;
  jobTitle: string;
}

export interface InviteUserInCompany {
  userId: string;
  role: CompanyRole;
  jobTitle: string;
  projectIds: string[];
}

export interface UserToInvite {
  username: string;
  userId: string;
}

export interface InvitedUserInCompanyInfo {
  notificationId: string;
  userId: string;
  username: string;
  role: CompanyRole;
  jobTitle: string;
  companyId: string;
}

export interface CompanyProjectInfo {
  id: string;
  name: string;
  code: string;
  active: boolean;
}

export interface NewCompanyProject {
  name: string;
  code: string;
}

export interface TimesheetActivityHours {
  companyId: string;
  companyName: string;
  projectId: string;
  projectName: string;
  activityId: string;
  activityName: string;
  notes: string;
  hours: number;
}

export interface TimesheetDay {
  userId: string;
  date: Moment;
  permitHours: number;
  workingType: TimesheetDayWorkType;
  activities: TimesheetActivityHours[];
}

export interface ProjectActivityInfo {
  id: string;
  name: string;
  description: string;
}

export interface NewProjectActivity {
  name: string;
  description: string;
}

export interface CalendarDay {
  date: Moment;
  dayName: string;
  dayNumber: number;
  isWeekend: boolean;
}

export interface TimesheetProjectInfo {
  companyId: string;
  companyName: string;
  projectId: string;
  projectName: string;
  activities: ProjectActivityInfo[];
}

export interface CorporateGroupInfo {
  groupId: string;
  name: string;
  companyIds: string[];
  companyNames: string[];
}

export interface AdminCorporateGroupInfo {
  id: string;
  name: string;
  active: boolean;
  ownerId: string | null;
}

export interface CorporateGroupCompanyInfo {
  name: string;
  id: string;
}

export interface CreateCorporateGroupParameters {
  name: string;
}

export interface EditCorporateGroupParameters {
  name: string;
  companyIds: string[];
}

export interface IdLabelElement {
  label: string;
  id: string;
}
