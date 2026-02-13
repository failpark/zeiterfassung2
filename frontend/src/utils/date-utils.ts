import { format, parseISO } from 'date-fns';
import { de } from 'date-fns/locale';

// Format date: 01.01.2023
export const formatDate = (date: string | Date): string => {
  const dateObj = typeof date === 'string' ? parseISO(date) : date;
  return format(dateObj, 'dd.MM.yyyy', { locale: de });
};

// Format time: 14:30
export const formatTime = (time: string): string => {
  // Extract hours and minutes from the time string (HH:MM:SS)
  const [hours, minutes] = time.split(':');
  return `${hours}:${minutes}`;
};

// Calculate duration between two times
export const calculateDuration = (start: string, end: string, pause?: string): number => {
  // Parse hours and minutes
  const [startHours, startMinutes] = start.split(':').map(Number);
  const [endHours, endMinutes] = end.split(':').map(Number);
  
  // Calculate total minutes
  let totalMinutes = (endHours * 60 + endMinutes) - (startHours * 60 + startMinutes);
  
  // Subtract pause if provided
  if (pause) {
    const [pauseHours, pauseMinutes] = pause.split(':').map(Number);
    totalMinutes -= (pauseHours * 60 + pauseMinutes);
  }
  
  // Convert to hours with 2 decimal places
  return parseFloat((totalMinutes / 60).toFixed(2));
};

// Format hours for display: 2.5 -> 2:30
export const formatHours = (hours: number): string => {
  const wholeHours = Math.floor(hours);
  const minutes = Math.round((hours - wholeHours) * 60);
  return `${wholeHours}:${minutes.toString().padStart(2, '0')}`;
};

// Get current date in ISO format (YYYY-MM-DD)
export const getCurrentDateISO = (): string => {
  return format(new Date(), 'yyyy-MM-dd');
};

// Get current time in ISO format (HH:MM:SS)
export const getCurrentTimeISO = (): string => {
  return format(new Date(), 'HH:mm:ss');
};