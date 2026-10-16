-- Applied once to an explicitly enabled, empty sample database volume.
CREATE TABLE customers (
  id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  name text NOT NULL,
  email text UNIQUE NOT NULL,
  active boolean NOT NULL DEFAULT true,
  created_at timestamptz NOT NULL DEFAULT now()
);
INSERT INTO customers (name, email) VALUES
 ('Amelia Brooks', 'amelia@example.com'),
 ('Oliver Chen', 'oliver@example.com'),
 ('Sofia Martinez', 'sofia@example.com');
CREATE TABLE orders (
 id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
 customer_id bigint NOT NULL REFERENCES customers(id),
 amount numeric(12,2) NOT NULL CHECK (amount >= 0),
 status text NOT NULL CHECK (status IN ('pending', 'completed', 'refunded'))
);
INSERT INTO orders (customer_id, amount, status) VALUES (1, 129.90, 'completed'), (2, 49.00, 'pending');
