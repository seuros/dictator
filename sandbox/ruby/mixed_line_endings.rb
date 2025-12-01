class NotificationsModel < ApplicationRecord
  belongs_to :user
  has_many :notification_recipients

  scope :unread, -> { where(read_at: nil) }
  scope :recent, -> { order(created_at: :desc) }

  validates :message, presence: true

  def mark_as_read
    update(read_at: Time.current)
  end

  def notify_user
    NotificationMailer.send_notification(self).deliver_later
  end
end

