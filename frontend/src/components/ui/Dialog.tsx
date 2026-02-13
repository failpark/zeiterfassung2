import React, { Fragment } from 'react';
import { Dialog as HeadlessDialog, Transition } from '@headlessui/react';
import { cn } from '../../utils/ui-utils';
import { XMarkIcon } from '@heroicons/react/24/outline';

interface DialogProps {
  isOpen: boolean;
  onClose: () => void;
  title?: string;
  description?: string;
  children?: React.ReactNode;
  className?: string;
  contentClassName?: string;
  titleClassName?: string;
  descriptionClassName?: string;
  showCloseButton?: boolean;
}

const Dialog = ({
  isOpen,
  onClose,
  title,
  description,
  children,
  className,
  contentClassName,
  titleClassName,
  descriptionClassName,
  showCloseButton = true,
}: DialogProps) => {
  return (
    <Transition appear show={isOpen} as={Fragment}>
      <HeadlessDialog
        as="div"
        className={cn('relative z-50', className)}
        onClose={onClose}
      >
        <Transition.Child
          as={Fragment}
          enter="ease-out duration-300"
          enterFrom="opacity-0"
          enterTo="opacity-100"
          leave="ease-in duration-200"
          leaveFrom="opacity-100"
          leaveTo="opacity-0"
        >
          <div className="fixed inset-0 bg-background/80 backdrop-blur-sm" />
        </Transition.Child>

        <div className="fixed inset-0 overflow-y-auto">
          <div className="flex min-h-full items-center justify-center p-4 text-center">
            <Transition.Child
              as={Fragment}
              enter="ease-out duration-300"
              enterFrom="opacity-0 scale-95"
              enterTo="opacity-100 scale-100"
              leave="ease-in duration-200"
              leaveFrom="opacity-100 scale-100"
              leaveTo="opacity-0 scale-95"
            >
              <HeadlessDialog.Panel
                className={cn(
                  'w-full max-w-md transform overflow-hidden rounded-lg bg-background p-6 text-left align-middle shadow-xl transition-all',
                  contentClassName
                )}
              >
                {showCloseButton && (
                  <button
                    type="button"
                    className="absolute right-4 top-4 rounded-sm opacity-70 ring-offset-background transition-opacity hover:opacity-100 focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2"
                    onClick={onClose}
                  >
                    <XMarkIcon className="h-4 w-4" />
                    <span className="sr-only">Close</span>
                  </button>
                )}

                {title && (
                  <HeadlessDialog.Title
                    as="h3"
                    className={cn(
                      'text-lg font-medium leading-6 text-foreground',
                      titleClassName
                    )}
                  >
                    {title}
                  </HeadlessDialog.Title>
                )}

                {description && (
                  <div className="mt-2">
                    <p
                      className={cn(
                        'text-sm text-muted-foreground',
                        descriptionClassName
                      )}
                    >
                      {description}
                    </p>
                  </div>
                )}

                <div className="mt-4">{children}</div>
              </HeadlessDialog.Panel>
            </Transition.Child>
          </div>
        </div>
      </HeadlessDialog>
    </Transition>
  );
};

export default Dialog;