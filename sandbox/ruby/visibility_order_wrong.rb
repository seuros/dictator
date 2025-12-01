class UsersService
  # Private methods defined BEFORE public methods (wrong order)

  private

  def validate_email(email)
    email.match?(/\A[\w+\-.]+@[a-z\d\-]+(\.[a-z]+)*\z/i)
  end

  def hash_password(password)
    BCrypt::Password.create(password)
  end

  def generate_reset_token
    SecureRandom.hex(32)
  end

  # Public methods come AFTER private (violation)

  public

  def create_user(name, email, password)
    return { error: 'Invalid email' } unless validate_email(email)
    return { error: 'Password too short' } if password.length < 8

    user = User.create(
      name: name,
      email: email,
      password_digest: hash_password(password)
    )

    { success: true, user: user }
  end

  def send_password_reset(email)
    user = User.find_by(email: email)
    return { error: 'User not found' } unless user.present?

    token = generate_reset_token
    user.update(reset_token: token, reset_token_expires_at: 1.hour.from_now)

    ResetEmailService.send(user, token)
    { success: true, message: 'Reset email sent' }
  end

  def authenticate(email, password)
    user = User.find_by(email: email)
    return nil unless user.present?

    if BCrypt::Password.new(user.password_digest) == password
      user
    else
      nil
    end
  end

  protected

  def notify_admins(message)
    Admin.all.each do |admin|
      AdminMailer.notify(admin, message).deliver_later
    end
  end
end
