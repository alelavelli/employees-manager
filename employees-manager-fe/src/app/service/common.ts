import { TimesheetDayWorkType } from '../types/enums';

export function timesheetDayWorkTypeToString(
  value: TimesheetDayWorkType
): string {
  switch (value) {
    case TimesheetDayWorkType.CompanyClosure:
      return 'Company closure';
    case TimesheetDayWorkType.DayOff:
      return 'Day off';
    case TimesheetDayWorkType.Holiday:
      return 'Holiday';
    case TimesheetDayWorkType.Office:
      return 'Office';
    case TimesheetDayWorkType.Remote:
      return 'Remote';
    case TimesheetDayWorkType.Sick:
      return 'Sick';
  }
}
