import { HttpClient } from '@angular/common/http';
import { Injectable } from '@angular/core';
import { Observable } from 'rxjs';
import { environment } from '../../environments/environment';
import { buildMocked } from './mock';
import {
  AdminPanelOverview,
  AdminPanelUserInfo,
  AppNotification,
  UserInCompanyInfo,
  CreateUserParameters,
  LoginResponse,
  UserData,
  CompanyInfo,
  CreateCompanyParameters,
  UserToInvite,
  InvitedUserInCompanyInfo,
  CompanyProjectInfo,
  ProjectActivityInfo,
  TimesheetDay,
  TimesheetActivityHours,
  TimesheetProjectInfo,
  CorporateGroupInfo,
  CreateCorporateGroupParameters,
  CorporateGroupCompanyInfo,
  EditCorporateGroupParameters,
  AdminCorporateGroupInfo,
} from '../types/model';
import {
  CompanyRole,
  CorporateGroupRole,
  NotificationType,
  TimesheetDayWorkType,
} from '../types/enums';
import { Moment } from 'moment';
import moment from 'moment';

const MOCKED = environment.mocked;
const API_URL = environment.apiHost + '/api';

@Injectable({
  providedIn: 'root',
})
export class ApiService {
  constructor(private httpClient: HttpClient) {}

  login(username: string, password: string): Observable<LoginResponse> {
    return MOCKED
      ? buildMocked({
          token: 'token',
          tokenType: 'Bearer',
        })
      : this.httpClient.post<LoginResponse>(API_URL + '/auth/login', {
          username,
          password,
        });
  }

  getUserData(): Observable<UserData> {
    return MOCKED
      ? buildMocked({
          id: 'my-id',
          username: 'my-username',
          name: 'my-name',
          surname: 'my-surname',
          email: 'my-email',
          platformAdmin: true,
          active: true,
        })
      : this.httpClient.get<UserData>(API_URL + '/auth/user');
  }

  getAdminPanelOverview(): Observable<AdminPanelOverview> {
    return MOCKED
      ? buildMocked({
          totalUsers: 10,
          totalAdmins: 2,
          totalActiveUsers: 8,
          totalInactiveUsers: 2,
          totalCompanies: 5,
        })
      : this.httpClient.get<AdminPanelOverview>(API_URL + '/admin/overview');
  }

  getAdminUsersInfo(): Observable<AdminPanelUserInfo[]> {
    return MOCKED
      ? buildMocked(
          [...Array(50).keys()].map((i) => ({
            id: `my-id-${i}`,
            username: `my-username-${i}`,
            name: `my-name-${i}`,
            surname: `my-surname-${i}`,
            email: `my-email-${i}`,
            platformAdmin: i % 2 === 0,
            active: i % 3 === 0,
            totalCompanies: i,
          }))
        )
      : this.httpClient.get<AdminPanelUserInfo[]>(API_URL + '/admin/user');
  }

  getAdminCorporateGroupsInfo(): Observable<AdminCorporateGroupInfo[]> {
    return MOCKED
      ? buildMocked(
          [...Array(50).keys()].map((i) => ({
            id: `my-id-${i}`,
            name: `my-corporate-group-name-${i}`,
            active: i % 3 === 0,
            ownerId: i % 2 === 0 ? 'this owner' : null,
          }))
        )
      : this.httpClient.get<AdminCorporateGroupInfo[]>(
          API_URL + '/admin/corporate-group'
        );
  }

  setPlatformAdminUser(userId: string): Observable<void> {
    return MOCKED
      ? buildMocked()
      : this.httpClient.post<void>(
          API_URL + `/admin/user/${userId}/platform-admin`,
          {}
        );
  }

  unsetPlatformAdminUser(userId: string): Observable<void> {
    return MOCKED
      ? buildMocked()
      : this.httpClient.delete<void>(
          API_URL + `/admin/user/${userId}/platform-admin`,
          {}
        );
  }

  setCorporateGroupOwner(
    corporateGroupId: string,
    userId: string
  ): Observable<void> {
    return MOCKED
      ? buildMocked()
      : this.httpClient.post<void>(
          API_URL +
            `/admin/corporate-group/${corporateGroupId}/owner/${userId}`,
          { role: CorporateGroupRole.Owner }
        );
  }

  setUserPassword(userId: string, newPassword: string): Observable<void> {
    return MOCKED
      ? buildMocked()
      : this.httpClient.patch<void>(
          API_URL + `/admin/user/${userId}/password`,
          { password: newPassword }
        );
  }

  activateUser(userId: string): Observable<void> {
    return MOCKED
      ? buildMocked()
      : this.httpClient.post<void>(
          API_URL + `/admin/user/${userId}/activate`,
          {}
        );
  }

  deactivateUser(userId: string): Observable<void> {
    return MOCKED
      ? buildMocked()
      : this.httpClient.delete<void>(
          API_URL + `/admin/user/${userId}/activate`,
          {}
        );
  }

  deleteUser(userId: string): Observable<void> {
    return MOCKED
      ? buildMocked()
      : this.httpClient.delete<void>(API_URL + `/admin/user/${userId}`, {});
  }

  activateCorporateGroup(corporateGroupId: string): Observable<void> {
    return MOCKED
      ? buildMocked()
      : this.httpClient.post<void>(
          API_URL + `/corporate-group/${corporateGroupId}/activate`,
          {}
        );
  }

  deactivateCorporateGroup(corporateGroupId: string): Observable<void> {
    return MOCKED
      ? buildMocked()
      : this.httpClient.delete<void>(
          API_URL + `/admin/corporate-group/${corporateGroupId}/activate`,
          {}
        );
  }

  deleteCorporateGroup(corporateGroupId: string): Observable<void> {
    return MOCKED
      ? buildMocked()
      : this.httpClient.delete<void>(
          API_URL + `/corporate-group/${corporateGroupId}`,
          {}
        );
  }

  createUser(user: CreateUserParameters): Observable<string> {
    return MOCKED
      ? buildMocked('new-user-id')
      : this.httpClient.post<string>(API_URL + '/admin/user', user);
  }

  getUnreadNotifications(): Observable<AppNotification[]> {
    return MOCKED
      ? buildMocked(
          [...Array(50).keys()].map((i) => ({
            id: `id-${i}`,
            notificationType:
              i % 2 == 0
                ? NotificationType.InviteAddCompany
                : NotificationType.InviteAddCompanyAnswer,
            message:
              i % 2 == 0
                ? `You have been invited to Company ${i}`
                : `The user accepted to join in Company ${i}`,
          }))
        )
      : this.httpClient.get<AppNotification[]>(API_URL + '/notification');
  }

  setNotificationAsRead(notificationId: string): Observable<void> {
    return MOCKED
      ? buildMocked()
      : this.httpClient.patch<void>(
          API_URL + `/notification/${notificationId}/read`,
          {}
        );
  }

  acceptInviteAddCompany(notificationId: string) {
    return MOCKED
      ? buildMocked()
      : this.httpClient.patch<void>(
          API_URL + `/notification/invite-add-company/${notificationId}`,
          {
            accept: true,
          }
        );
  }

  declineInviteAddCompany(notificationId: string) {
    return MOCKED
      ? buildMocked()
      : this.httpClient.patch<void>(
          API_URL + `/notification/invite-add-company/${notificationId}`,
          {
            accept: false,
          }
        );
  }

  getUserCompanies(): Observable<CompanyInfo[]> {
    return MOCKED
      ? buildMocked(
          [...Array(50).keys()].map((i) => ({
            id: `id-${i}`,
            name: `company-${i}`,
            active: i % 3 === 0,
            totalUsers: i * 2,
            role: [CompanyRole.Owner, CompanyRole.Admin, CompanyRole.User][
              i % 3
            ],
          }))
        )
      : this.httpClient.get<CompanyInfo[]>(API_URL + '/company');
  }

  getUserCorporateGroups(): Observable<CorporateGroupInfo[]> {
    return MOCKED
      ? buildMocked(
          [...Array(5).keys()].map((i) => ({
            groupId: `corporate-group-id-${i}`,
            name: `corporate-group-name-${i}`,
            companyIds: [
              `company-id-${i + 1}`,
              `company-id-${i + 2}`,
              `company-id-${i + 3}`,
              `company-id-${i + 4}`,
              `company-id-${i + 5}`,
            ],
            companyNames: [
              `company-name-${i + 1}`,
              `company-name-${i + 2}`,
              `company-name-${i + 3}`,
              `company-name-${i + 4}`,
              `company-name-${i + 5}`,
            ],
          }))
        )
      : this.httpClient.get<CorporateGroupInfo[]>(API_URL + '/corporate-group');
  }

  getEligibleCompaniesForCorporateGroup(): Observable<
    CorporateGroupCompanyInfo[]
  > {
    return MOCKED
      ? buildMocked(
          [...Array(5).keys()].map((i) => ({
            name: `company-name-${i}`,
            id: `company-id-${i}`,
          }))
        )
      : this.httpClient.get<CorporateGroupCompanyInfo[]>(
          API_URL + '/corporate-group/eligible-company'
        );
  }

  editCorporateGroup(
    corporateGroupId: string,
    updatedGroup: EditCorporateGroupParameters
  ): Observable<void> {
    return MOCKED
      ? buildMocked()
      : this.httpClient.patch<void>(
          API_URL + `/corporate-group/${corporateGroupId}`,
          updatedGroup
        );
  }

  createCorporateGroup(
    newGroup: CreateCorporateGroupParameters
  ): Observable<void> {
    return MOCKED
      ? buildMocked()
      : this.httpClient.post<void>(
          API_URL + `/admin/corporate-group`,
          newGroup
        );
  }

  getUsersInCompany(companyId: string): Observable<UserInCompanyInfo[]> {
    return MOCKED
      ? buildMocked(
          [...Array(50).keys()].map((i) => ({
            userId: `user-id-${i}`,
            userName: `name-${i}`,
            userSurname: `surname-${i}`,
            userUsername: `username-${i}`,
            companyId: `company-id-${i}`,
            jobTitle: `job-title-${i}`,
            role: i % 3 === 0 ? CompanyRole.Admin : CompanyRole.User,
            job_title: `job-title-${i}`,
            managementTeam: i % 2 === 0,
          }))
        )
      : this.httpClient.get<UserInCompanyInfo[]>(
          API_URL + `/company/${companyId}/user`
        );
  }

  getPendingUsersInCompany(
    companyId: string
  ): Observable<InvitedUserInCompanyInfo[]> {
    return MOCKED
      ? buildMocked(
          [...Array(50).keys()].map((i) => ({
            userId: `user-id-${i}`,
            notificationId: `notification-id-${i}`,
            username: `name-${i}`,
            companyId: `company-id-${i}`,
            jobTitle: `job-title-${i}`,
            role: i % 3 === 0 ? CompanyRole.Admin : CompanyRole.User,
          }))
        )
      : this.httpClient.get<InvitedUserInCompanyInfo[]>(
          API_URL + `/company/${companyId}/pending-user`
        );
  }

  createCompany(company: CreateCompanyParameters): Observable<string> {
    return MOCKED
      ? buildMocked('new-company-id')
      : this.httpClient.post<string>(API_URL + '/company', company);
  }

  changeUserCompanyRole(
    companyId: string,
    userId: string,
    role: CompanyRole
  ): Observable<void> {
    return MOCKED
      ? buildMocked()
      : this.httpClient.patch<void>(API_URL + `/company/${companyId}/role`, {
          userId: userId,
          role: role,
        });
  }

  changeUserJobTitle(
    companyId: string,
    userId: string,
    jobTitle: string
  ): Observable<void> {
    return MOCKED
      ? buildMocked()
      : this.httpClient.patch<void>(
          API_URL + `/company/${companyId}/job-title`,
          {
            userId: userId,
            jobTitle: jobTitle,
          }
        );
  }

  setUserCompanyManager(companyId: string, userId: string): Observable<void> {
    return MOCKED
      ? buildMocked()
      : this.httpClient.patch<void>(API_URL + `/company/${companyId}/manager`, {
          userId: userId,
          manager: true,
        });
  }

  unsetUserCompanyManager(companyId: string, userId: string): Observable<void> {
    return MOCKED
      ? buildMocked()
      : this.httpClient.patch<void>(API_URL + `/company/${companyId}/manager`, {
          userId: userId,
          manager: false,
        });
  }

  removeUserFromCompany(companyId: string, userId: string): Observable<void> {
    return MOCKED
      ? buildMocked()
      : this.httpClient.delete<void>(
          API_URL + `/company/${companyId}/user/${userId}`,
          {}
        );
  }

  getUsersToInvite(companyId: string): Observable<UserToInvite[]> {
    return MOCKED
      ? buildMocked(
          [...Array(20).keys()].map((i) => ({
            userId: `user-id-${i}`,
            username: `name-${i}`,
          }))
        )
      : this.httpClient.get<UserToInvite[]>(
          API_URL + `/company/${companyId}/user-to-invite`
        );
  }

  inviteUserInCompany(
    companyId: string,
    userId: string,
    role: CompanyRole,
    jobTitle: string,
    projectIds: string[]
  ): Observable<void> {
    return MOCKED
      ? buildMocked()
      : this.httpClient.post<void>(
          API_URL + `/company/${companyId}/invite-user`,
          {
            userId: userId,
            role: role,
            jobTitle: jobTitle,
            projectIds: projectIds,
          }
        );
  }

  cancelInvitation(
    companyId: string,
    notificationId: string
  ): Observable<void> {
    return MOCKED
      ? buildMocked()
      : this.httpClient.delete<void>(
          API_URL + `/company/${companyId}/invite-user/${notificationId}`,
          {}
        );
  }

  getCompanyProjects(companyId: string): Observable<CompanyProjectInfo[]> {
    return MOCKED
      ? buildMocked(
          [...Array(20).keys()].map((i) => ({
            id: `project-${i}`,
            name: `project-name-${i}`,
            code: `code-${i}`,
            active: i % 2 === 0,
          }))
        )
      : this.httpClient.get<CompanyProjectInfo[]>(
          API_URL + `/company/${companyId}/project`
        );
  }

  deleteCompanyProject(companyId: string, projectId: string): Observable<void> {
    return MOCKED
      ? buildMocked()
      : this.httpClient.delete<void>(
          API_URL + `/company/${companyId}/project/${projectId}`
        );
  }

  createCompanyProject(
    companyId: string,
    name: string,
    code: string
  ): Observable<void> {
    return MOCKED
      ? buildMocked()
      : this.httpClient.post<void>(API_URL + `/company/${companyId}/project`, {
          name: name,
          code: code,
        });
  }

  createProjectActivity(
    companyId: string,
    name: string,
    description: string
  ): Observable<void> {
    return MOCKED
      ? buildMocked()
      : this.httpClient.post<void>(API_URL + `/company/${companyId}/activity`, {
          name: name,
          description: description,
        });
  }

  editCompanyProject(
    companyId: string,
    projectId: string,
    name: string,
    code: string,
    active: boolean
  ): Observable<void> {
    return MOCKED
      ? buildMocked()
      : this.httpClient.patch<void>(
          API_URL + `/company/${companyId}/project/${projectId}`,
          {
            name: name,
            code: code,
            active: active,
          }
        );
  }

  getCompanyProjectActivities(
    companyId: string
  ): Observable<ProjectActivityInfo[]> {
    return MOCKED
      ? buildMocked(
          [...Array(10).keys()].map((i) => ({
            name: `activity-${i}`,
            id: `id-${i}`,
            description: `this description is very long and needs to be handled very carefully. Do you understand?`,
          }))
        )
      : this.httpClient.get<ProjectActivityInfo[]>(
          API_URL + `/company/${companyId}/activity`
        );
  }

  editProjectActivity(
    companyId: string,
    activityId: string,
    name: string,
    description: string
  ): Observable<void> {
    return MOCKED
      ? buildMocked()
      : this.httpClient.patch<void>(
          API_URL + `/company/${companyId}/activity/${activityId}`,
          {
            name: name,
            description: description,
          }
        );
  }

  deleteActivity(companyId: string, activityId: string): Observable<void> {
    return MOCKED
      ? buildMocked()
      : this.httpClient.delete<void>(
          API_URL + `/company/${companyId}/activity/${activityId}`
        );
  }

  getCompanyProjectAllocationsByProject(
    companyId: string,
    projectId: string
  ): Observable<string[]> {
    return MOCKED
      ? buildMocked([...Array(5).keys()].map((i) => `user-id-${i}`))
      : this.httpClient.get<string[]>(
          API_URL + `/company/${companyId}/project-allocation/${projectId}`
        );
  }

  getCompanyProjectAllocationsByUser(
    companyId: string,
    userId: string
  ): Observable<string[]> {
    return MOCKED
      ? buildMocked([...Array(5).keys()].map((i) => `project-${i}`))
      : this.httpClient.get<string[]>(
          API_URL + `/company/${companyId}/user-allocation/${userId}`
        );
  }

  updateCompanyProjectAllocationsByProject(
    companyId: string,
    projectId: string,
    userIds: string[]
  ): Observable<void> {
    return MOCKED
      ? buildMocked()
      : this.httpClient.patch<void>(
          API_URL + `/company/${companyId}/project-allocation/${projectId}`,
          { userIds: userIds }
        );
  }

  updateCompanyProjectAllocationsByUser(
    companyId: string,
    userId: string,
    projectIds: string[]
  ): Observable<void> {
    return MOCKED
      ? buildMocked()
      : this.httpClient.patch<void>(
          API_URL + `/company/${companyId}/user-allocation/${userId}`,
          { projectIds: projectIds }
        );
  }

  getCompanyProjectActivitiesByProject(
    companyId: string,
    projectId: string
  ): Observable<string[]> {
    return MOCKED
      ? buildMocked([...Array(5).keys()].map((i) => `id-${i}`))
      : this.httpClient.get<string[]>(
          API_URL + `/company/${companyId}/project-activity/${projectId}`
        );
  }

  getCompanyProjectActivitiesByActivity(
    companyId: string,
    activityId: string
  ): Observable<string[]> {
    return MOCKED
      ? buildMocked([...Array(5).keys()].map((i) => `project-${i}`))
      : this.httpClient.get<string[]>(
          API_URL + `/company/${companyId}/activity-assignment/${activityId}`
        );
  }

  updateCompanyProjectActivitiesByProject(
    companyId: string,
    projectId: string,
    activityIds: string[]
  ): Observable<void> {
    return MOCKED
      ? buildMocked()
      : this.httpClient.patch<void>(
          API_URL + `/company/${companyId}/project-activity/${projectId}`,
          { activityIds: activityIds }
        );
  }

  updateCompanyProjectActivitiesByActivity(
    companyId: string,
    activityId: string,
    projectIds: string[]
  ): Observable<void> {
    return MOCKED
      ? buildMocked()
      : this.httpClient.patch<void>(
          API_URL + `/company/${companyId}/activity-assignment/${activityId}`,
          { projectIds: projectIds }
        );
  }

  getTimesheetDays(
    userId: string,
    year: number,
    month: number
  ): Observable<TimesheetDay[]> {
    return MOCKED
      ? buildMocked(
          [...Array(10).keys()].map((i) => ({
            userId: `user-id-${i}`,
            date: moment(`2025-${1}-${i}`, 'YYYY-MM-DD'),
            permitHours: 2,
            workingType: TimesheetDayWorkType.Office,
            activities: [
              {
                companyId: `company-id-${i}`,
                companyName: `company-name-${i}`,
                projectId: `project-id-${i}`,
                projectName: `project-name-${i}`,
                activityId: `activity-id-${i}`,
                activityName: `activity-name-${i}`,
                notes: `notes-${i}`,
                hours: i,
              },
              {
                companyId: `company-id-${i}`,
                companyName: `company-name-${i}`,
                projectId: `second-project-id-${i}`,
                projectName: `second-project-name-${i}`,
                activityId: `second-activity-id-${i}`,
                activityName: `second-activity-name-${i}`,
                notes: `notes-${i}`,
                hours: i,
              },
            ] as TimesheetActivityHours[],
          }))
        )
      : this.httpClient.get<TimesheetDay[]>(
          API_URL + `/user/${userId}/timesheet-day`,
          { params: { year: year.toString(), month: month.toString() } }
        );
  }

  getUserProjectsForTimesheet(
    userId: string
  ): Observable<TimesheetProjectInfo[]> {
    return MOCKED
      ? buildMocked(
          [...Array(10).keys()].map((i) => ({
            companyId: i < 5 ? `company-id-1` : 'company-id-3',
            companyName: i < 5 ? `company-name-1` : 'company-name-3',
            projectId: i % 2 == 0 ? `project-id-1` : 'project-id-3',
            projectName: i % 2 == 0 ? `project-name-1` : 'project-name-3',
            activities: [
              {
                id: `activity-id-1`,
                name: `activity-name-1`,
                description: `description-${i}`,
              },
              {
                id: `activity-id-3`,
                name: `activity-name-3`,
                description: `description-${i}`,
              },
            ],
          }))
        )
      : this.httpClient.get<TimesheetProjectInfo[]>(
          API_URL + `/user/${userId}/timesheet-project`
        );
  }

  createTimesheetDay(
    userId: string,
    day: Moment,
    permitHours: number,
    dayType: TimesheetDayWorkType,
    activities: TimesheetActivityHours[]
  ): Observable<void> {
    return MOCKED
      ? buildMocked()
      : this.httpClient.post<void>(API_URL + `/user/${userId}/timesheet-day`, {
          date: day.toISOString(),
          permitHours: permitHours,
          workingType: dayType,
          activities: activities,
        });
  }

  exportUserTimesheet(year: number, month: number): Observable<Blob> {
    return MOCKED
      ? buildMocked(new Blob())
      : this.httpClient.get<Blob>(API_URL + `/user/timesheet-export`, {
          params: { year: year.toString(), month: month.toString() },
          responseType: 'blob' as 'json',
        });
  }
}
