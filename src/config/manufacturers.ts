/**
 * Manufacturer definitions for board categorization
 * Vendor configuration is loaded from vendors.json
 * Logos are at cache.armbian.com/images/vendors/300/{id}.png
 */

import vendorsData from './vendors.json';

export interface ManufacturerConfig {
  name: string;
  color: string;
}

interface VendorConfig {
  prefixes: string[];
  name?: string;
}

const VENDORS: Record<string, VendorConfig> = vendorsData.vendors;
const DEFAULT_COLOR = '#64748b';

// Build prefix lookup map for fast matching
const PREFIX_TO_VENDOR: Array<[string, string]> = [];
for (const [vendorId, config] of Object.entries(VENDORS)) {
  for (const prefix of config.prefixes) {
    PREFIX_TO_VENDOR.push([prefix, vendorId]);
  }
}
// Sort by prefix length descending so longer prefixes match first
PREFIX_TO_VENDOR.sort((a, b) => b[0].length - a[0].length);

/**
 * Get display name for a vendor
 */
function getDisplayName(vendorId: string): string {
  if (vendorId === 'other') return 'Other Boards';
  const vendor = VENDORS[vendorId];
  if (vendor?.name) return vendor.name;
  return vendorId.split('-').map(w => w.charAt(0).toUpperCase() + w.slice(1)).join(' ');
}

/**
 * Get manufacturer config for a vendor
 */
export function getManufacturerConfig(vendorId: string): ManufacturerConfig {
  return { name: getDisplayName(vendorId), color: DEFAULT_COLOR };
}

// Dynamic proxy for backward compatibility
export const MANUFACTURERS: Record<string, ManufacturerConfig> = new Proxy(
  {} as Record<string, ManufacturerConfig>,
  { get: (_, key: string) => getManufacturerConfig(key) }
);

/**
 * Check if a vendor has a logo
 */
export function vendorHasLogo(vendorId: string): boolean {
  return vendorId in VENDORS;
}

/**
 * Get vendor ID from board display name
 */
export function getManufacturer(_slug: string, displayName?: string): string {
  if (displayName) {
    for (const [prefix, vendorId] of PREFIX_TO_VENDOR) {
      if (displayName.startsWith(prefix)) {
        return vendorId;
      }
    }
  }
  return 'other';
}

/**
 * Get vendor logo URL
 */
export function getVendorLogoUrl(vendorId: string): string {
  return `https://cache.armbian.com/images/vendors/300/${vendorId}.png`;
}
