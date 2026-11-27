-- Up Migration: Adjust chk_sales_qty_valid constraint on sales table
-- Allow quantity >= 0.00 when status = 'Order' and requested_quantity > 0.00
-- Enforce quantity > 0.00 when status = 'Realized' and requested_quantity >= 0.00

ALTER TABLE sales DROP CONSTRAINT IF EXISTS chk_sales_qty_valid;

-- Normalize any historical Order rows where requested_quantity was 0.00 but quantity > 0.00
UPDATE sales
SET requested_quantity = quantity
WHERE status = 'Order' AND requested_quantity <= 0.00 AND quantity > 0.00;

ALTER TABLE sales ADD CONSTRAINT chk_sales_qty_valid CHECK (
    (status = 'Order' AND requested_quantity > 0.00 AND quantity >= 0.00)
    OR
    (status = 'Realized' AND quantity > 0.00 AND requested_quantity >= 0.00)
);
