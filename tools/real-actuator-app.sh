#!/usr/bin/env bash
# Builds a Laravel app with laravel-modules, laravel-actuator and laravel-metrics in a new folder.
# The job "Real actuator" in .github/workflows/ci.yml runs it, and so can a person on a Mac.
# It follows fixtures/actuator/README.md with versions as ranges, so a new release is caught.
# It needs PHP 8.4 with pdo_sqlite, and Composer.
#
#   tools/real-actuator-app.sh <dir>
#
# <dir> must not exist. The app has two modules, Blog and Billing, each with one health
# indicator, one web route with a parameter and one job. The files in tools/real-actuator/
# are copied over the files the generators wrote. Serve it with `php artisan serve`.
set -euo pipefail

if [ "$#" -ne 1 ]; then
  echo "usage: $0 <dir>" >&2
  exit 2
fi
dir=$1
if [ -e "$dir" ]; then
  echo "$dir exists. Name a folder that does not exist." >&2
  exit 2
fi
files=$(cd "$(dirname "$0")/real-actuator" && pwd)

composer create-project laravel/laravel "$dir" '^13.0' --no-interaction --prefer-dist
cd "$dir"
# The package autoloads modules through wikimedia/composer-merge-plugin.
composer config --no-plugins allow-plugins.wikimedia/composer-merge-plugin true
composer config --json extra.merge-plugin '{"include": ["Modules/*/composer.json"]}'
composer require 'nwidart/laravel-modules:^13.4' 'nwidart/laravel-actuator:^1.3' \
  'nwidart/laravel-metrics:^1.1' -W --no-interaction
php artisan vendor:publish --provider="Nwidart\Modules\LaravelModulesServiceProvider"
php artisan vendor:publish --tag=actuator-config
php artisan vendor:publish --tag=metrics-config

php artisan module:make Blog Billing
php artisan module:make-health-indicator StorageHealthIndicator Blog
php artisan module:make-health-indicator GatewayHealthIndicator Billing
php artisan module:make-job PublishPost Blog
php artisan module:make-job ChargeInvoice Billing
php artisan module:make-controller PostController Blog
php artisan module:make-controller InvoiceController Billing
cp -R "$files/." .

# Each module registers its indicator in the provider the generator wrote.
register() {
  local provider=Modules/$1/app/Providers/$1ServiceProvider.php
  local line="protected array \$healthIndicators = ['$2' => \\\\Modules\\\\$1\\\\Health\\\\$3::class];"
  sed -i.bak "s|// protected array \$healthIndicators = \[\];|$line|" "$provider"
  rm "$provider.bak"
  if ! grep -qF "'$2' => \\Modules\\$1\\Health\\$3::class" "$provider"; then
    echo "$provider has no line for \$healthIndicators. The generator changed." >&2
    exit 1
  fi
}
register Blog storage StorageHealthIndicator
register Billing gateway GatewayHealthIndicator

# The endpoints Mortise reads. Never env.
include="'include' => ['health', 'info', 'mappings', 'scheduledtasks', 'prometheus'],"
sed -i.bak "s|'include' => \['health'\],|$include|" config/actuator.php
rm config/actuator.php.bak
if ! grep -qF "$include" config/actuator.php; then
  echo "config/actuator.php has no exposure line to change. The package changed." >&2
  exit 1
fi

cat >> .env <<'EOF'

ACTUATOR_HEALTH_SHOW_DETAILS=always
METRICS_ENABLED=true
METRICS_STORE=cache
METRICS_CACHE_STORE=file
EOF

composer dump-autoload
php artisan actuator:metrics:clear
composer show --direct
