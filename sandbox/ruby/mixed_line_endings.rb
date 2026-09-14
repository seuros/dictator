# Intentional: alternating CRLF and LF line endings
class LineEndings
  def unix
    :lf
  end

  def dos
    :crlf
  end
end
