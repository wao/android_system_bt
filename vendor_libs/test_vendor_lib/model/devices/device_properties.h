/*
 * Copyright 2015 The Android Open Source Project
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

#pragma once

#include <array>
#include <cstdint>
#include <string>
#include <vector>

#include "hci/address.h"
#include "hci/hci_packets.h"
#include "os/log.h"

namespace test_vendor_lib {

using ::bluetooth::hci::Address;
using ::bluetooth::hci::ClassOfDevice;
using ::bluetooth::hci::EventCode;

class DeviceProperties {
 public:
  explicit DeviceProperties(const std::string& file_name = "");

  // Access private configuration data

  // Specification Version 4.2, Volume 2, Part E, Section 7.4.1
  const std::vector<uint8_t>& GetVersionInformation() const;

  // Specification Version 4.2, Volume 2, Part E, Section 7.4.2
  const std::array<uint8_t, 64>& GetSupportedCommands() const {
    return supported_commands_;
  }

  void SetSupportedCommands(const std::array<uint8_t, 64>& commands) {
    if (!use_supported_commands_from_file_) {
      supported_commands_ = commands;
    }
  }

  // Specification Version 4.2, Volume 2, Part E, Section 7.4.3
  uint64_t GetSupportedFeatures() const {
    return extended_features_[0];
  }

  void SetExtendedFeatures(uint64_t features, uint8_t page_number) {
    ASSERT(page_number < extended_features_.size());
    extended_features_[page_number] = features;
  }

  bool GetSecureSimplePairingSupported() const {
    uint64_t ssp_bit = 0x1;
    return extended_features_[1] & ssp_bit;
  }

  void SetSecureSimplePairingSupport(bool supported) {
    uint64_t ssp_bit = 0x1;
    extended_features_[1] &= ~ssp_bit;
    if (supported) {
      extended_features_[1] = extended_features_[1] | ssp_bit;
    }
  }

  void SetLeHostSupport(bool le_supported) {
    uint64_t le_bit = 0x2;
    extended_features_[1] &= ~le_bit;
    if (le_supported) {
      extended_features_[1] = extended_features_[1] | le_bit;
    }
  }

  void SetSecureConnections(bool supported) {
    uint64_t secure_bit = 0x8;
    extended_features_[1] &= ~secure_bit;
    if (supported) {
      extended_features_[1] = extended_features_[1] | secure_bit;
    }
  }

  // Specification Version 4.2, Volume 2, Part E, Section 7.4.4
  uint8_t GetExtendedFeaturesMaximumPageNumber() const {
    return extended_features_.size() - 1;
  }

  uint64_t GetExtendedFeatures(uint8_t page_number) const {
    ASSERT(page_number < extended_features_.size());
    return extended_features_[page_number];
  }

  // Specification Version 4.2, Volume 2, Part E, Section 7.4.5
  uint16_t GetAclDataPacketSize() const {
    return acl_data_packet_size_;
  }

  uint8_t GetSynchronousDataPacketSize() const {
    return sco_data_packet_size_;
  }

  uint8_t GetEncryptionKeySize() const { return encryption_key_size_; }

  uint16_t GetVoiceSetting() const { return voice_setting_; }

  void SetVoiceSetting(uint16_t voice_setting) {
    voice_setting_ = voice_setting;
  }

  uint16_t GetConnectionAcceptTimeout() const {
    return connection_accept_timeout_;
  }

  void SetConnectionAcceptTimeout(uint16_t connection_accept_timeout) {
    connection_accept_timeout_ = connection_accept_timeout;
  }

  uint16_t GetTotalNumAclDataPackets() const {
    return num_acl_data_packets_;
  }

  uint16_t GetTotalNumSynchronousDataPackets() const {
    return num_sco_data_packets_;
  }

  const Address& GetAddress() const {
    return address_;
  }

  void SetAddress(const Address& address) {
    address_ = address;
  }

  // Specification Version 4.2, Volume 2, Part E, Section 7.4.8
  const std::vector<uint8_t>& GetSupportedCodecs() const {
    return supported_codecs_;
  }

  const std::vector<uint32_t>& GetVendorSpecificCodecs() const {
    return vendor_specific_codecs_;
  }

  uint8_t GetVersion() const {
    return version_;
  }

  uint16_t GetRevision() const {
    return revision_;
  }

  uint8_t GetLmpPalVersion() const {
    return lmp_pal_version_;
  }

  uint16_t GetLmpPalSubversion() const {
    return lmp_pal_subversion_;
  }

  uint16_t GetManufacturerName() const {
    return manufacturer_name_;
  }

  uint8_t GetAuthenticationEnable() const {
    return authentication_enable_;
  }

  void SetAuthenticationEnable(uint8_t enable) {
    authentication_enable_ = enable;
  }

  ClassOfDevice GetClassOfDevice() const {
    return class_of_device_;
  }

  void SetClassOfDevice(uint8_t b0, uint8_t b1, uint8_t b2) {
    class_of_device_.cod[0] = b0;
    class_of_device_.cod[1] = b1;
    class_of_device_.cod[2] = b2;
  }

  void SetClassOfDevice(uint32_t class_of_device) {
    class_of_device_.cod[0] = class_of_device & 0xff;
    class_of_device_.cod[1] = (class_of_device >> 8) & 0xff;
    class_of_device_.cod[2] = (class_of_device >> 16) & 0xff;
  }

  void SetName(const std::vector<uint8_t>& name) {
    name_.fill(0);
    for (size_t i = 0; i < 248 && i < name.size(); i++) {
      name_[i] = name[i];
    }
  }

  const std::array<uint8_t, 248>& GetName() const { return name_; }

  void SetExtendedInquiryData(const std::vector<uint8_t>& eid) {
    extended_inquiry_data_ = eid;
  }

  const std::vector<uint8_t>& GetExtendedInquiryData() const {
    return extended_inquiry_data_;
  }

  uint8_t GetPageScanRepetitionMode() const {
    return page_scan_repetition_mode_;
  }

  void SetPageScanRepetitionMode(uint8_t mode) {
    page_scan_repetition_mode_ = mode;
  }

  uint16_t GetClockOffset() const {
    return clock_offset_;
  }

  void SetClockOffset(uint16_t offset) {
    clock_offset_ = offset;
  }

  uint64_t GetEventMask() const { return event_mask_; }

  void SetEventMask(uint64_t mask) { event_mask_ = mask; }

  bool IsUnmasked(EventCode event) const {
    uint64_t bit = UINT64_C(1) << (static_cast<uint8_t>(event) - 1);
    return (event_mask_ & bit) != 0;
  }

  // Low-Energy functions
  const Address& GetLeAddress() const {
    return le_address_;
  }

  void SetLeAddress(const Address& address) {
    le_address_ = address;
  }

  uint8_t GetLeAddressType() const {
    return le_address_type_;
  }

  void SetLeAddressType(uint8_t addr_type) {
    le_address_type_ = addr_type;
  }

  uint8_t GetLeAdvertisementType() const {
    return le_advertisement_type_;
  }

  uint16_t GetLeAdvertisingIntervalMin() const {
    return le_advertising_interval_min_;
  }

  uint16_t GetLeAdvertisingIntervalMax() const {
    return le_advertising_interval_max_;
  }

  uint8_t GetLeAdvertisingOwnAddressType() const {
    return le_advertising_own_address_type_;
  }

  uint8_t GetLeAdvertisingPeerAddressType() const {
    return le_advertising_peer_address_type_;
  }

  Address GetLeAdvertisingPeerAddress() const {
    return le_advertising_peer_address_;
  }

  uint8_t GetLeAdvertisingChannelMap() const {
    return le_advertising_channel_map_;
  }

  uint8_t GetLeAdvertisingFilterPolicy() const {
    return le_advertising_filter_policy_;
  }

  void SetLeAdvertisingParameters(uint16_t interval_min, uint16_t interval_max, uint8_t ad_type,
                                  uint8_t own_address_type, uint8_t peer_address_type, Address peer_address,
                                  uint8_t channel_map, uint8_t filter_policy) {
    le_advertisement_type_ = ad_type;
    le_advertising_interval_min_ = interval_min;
    le_advertising_interval_max_ = interval_max;
    le_advertising_own_address_type_ = own_address_type;
    le_advertising_peer_address_type_ = peer_address_type;
    le_advertising_peer_address_ = peer_address;
    le_advertising_channel_map_ = channel_map;
    le_advertising_filter_policy_ = filter_policy;
  }

  void SetLeAdvertisementType(uint8_t ad_type) {
    le_advertisement_type_ = ad_type;
  }

  void SetLeAdvertisement(const std::vector<uint8_t>& ad) {
    le_advertisement_ = ad;
  }

  const std::vector<uint8_t>& GetLeAdvertisement() const {
    return le_advertisement_;
  }

  void SetLeScanResponse(const std::vector<uint8_t>& response) {
    le_scan_response_ = response;
  }

  const std::vector<uint8_t>& GetLeScanResponse() const {
    return le_scan_response_;
  }

  // Specification Version 4.2, Volume 2, Part E, Section 7.8.2
  uint16_t GetLeDataPacketLength() const {
    return le_data_packet_length_;
  }

  uint8_t GetTotalNumLeDataPackets() const {
    return num_le_data_packets_;
  }

  // Specification Version 4.2, Volume 2, Part E, Section 7.8.3
  uint64_t GetLeSupportedFeatures() const {
    return le_supported_features_;
  }

  // Specification Version 5.2, Volume 4, Part E, Section 7.8.6
  int8_t GetLeAdvertisingPhysicalChannelTxPower() const {
    return le_advertising_physical_channel_tx_power_;
  }

  void SetLeSupportedFeatures(uint64_t features) {
    le_supported_features_ = features;
  }

  bool GetLeEventSupported(bluetooth::hci::SubeventCode subevent_code) const {
    return le_event_mask_ & (1u << static_cast<uint64_t>(subevent_code));
  }

  uint64_t GetLeEventMask() const { return le_event_mask_; }

  void SetLeEventMask(uint64_t mask) { le_event_mask_ = mask; }

  // Specification Version 4.2, Volume 2, Part E, Section 7.8.14
  uint8_t GetLeConnectListSize() const { return le_connect_list_size_; }

  // Specification Version 4.2, Volume 2, Part E, Section 7.8.27
  uint64_t GetLeSupportedStates() const {
    return le_supported_states_;
  }

  // Specification Version 4.2, Volume 2, Part E, Section 7.8.41
  uint8_t GetLeResolvingListSize() const { return le_resolving_list_size_; }

  // Vendor-specific commands
  const std::vector<uint8_t>& GetLeVendorCap() const {
    return le_vendor_cap_;
  }

 private:
  // Classic
  uint16_t acl_data_packet_size_;
  uint8_t sco_data_packet_size_;
  uint16_t num_acl_data_packets_;
  uint16_t num_sco_data_packets_;
  uint8_t version_;
  uint16_t revision_;
  uint8_t lmp_pal_version_;
  uint16_t manufacturer_name_;
  uint16_t lmp_pal_subversion_;
  uint64_t supported_features_{};
  uint64_t event_mask_{0x00001fffffffffff};
  uint8_t authentication_enable_{};
  std::vector<uint8_t> supported_codecs_;
  std::vector<uint32_t> vendor_specific_codecs_;
  std::array<uint8_t, 64> supported_commands_;
  std::vector<uint64_t> extended_features_{{0x875b3fd8fe8ffeff, 0x04}};
  ClassOfDevice class_of_device_{{0, 0, 0}};
  std::vector<uint8_t> extended_inquiry_data_;
  std::array<uint8_t, 248> name_{};
  Address address_{};
  uint8_t page_scan_repetition_mode_{};
  uint16_t clock_offset_{};
  uint8_t encryption_key_size_{10};
  uint16_t voice_setting_{0x0060};
  uint16_t connection_accept_timeout_{0x7d00};
  bool use_supported_commands_from_file_ = false;

  // Low Energy
  uint16_t le_data_packet_length_;
  uint8_t num_le_data_packets_;
  uint8_t le_connect_list_size_;
  uint8_t le_resolving_list_size_;
  uint64_t le_supported_features_{0x075b3fd8fe8ffeff};
  int8_t le_advertising_physical_channel_tx_power_{0x00};
  uint64_t le_supported_states_;
  uint64_t le_event_mask_{0x01f};
  std::vector<uint8_t> le_vendor_cap_;
  Address le_address_{};
  uint8_t le_address_type_{};

  uint16_t le_advertising_interval_min_{};
  uint16_t le_advertising_interval_max_{};
  uint8_t le_advertising_own_address_type_{};
  uint8_t le_advertising_peer_address_type_{};
  Address le_advertising_peer_address_{};
  uint8_t le_advertising_channel_map_{};
  uint8_t le_advertising_filter_policy_{};
  uint8_t le_advertisement_type_{};
  std::vector<uint8_t> le_advertisement_;
  std::vector<uint8_t> le_scan_response_;
};

}  // namespace test_vendor_lib
