// Copyright 2020 Contributors to the Parsec project.
// SPDX-License-Identifier: Apache-2.0
use num_traits::ToPrimitive;
use std::convert::TryFrom;
use tss_esapi::constants::algorithm::HashingAlgorithm;
use tss_esapi::constants::tss::TPM2_ALG_LAST;
use tss_esapi::structures::{PcrSelectSize, PcrSelectionList, PcrSelectionListBuilder, PcrSlot};
use tss_esapi::tss2_esys::{TPM2_ALG_ID, TPML_PCR_SELECTION};

mod test_pcr_selection_list_builder {
    use super::*;

    #[test]
    fn test_one_selection() {
        let pcr_selection_list = PcrSelectionListBuilder::new()
            .with_selection(
                HashingAlgorithm::Sha256,
                &[PcrSlot::Slot0, PcrSlot::Slot8, PcrSlot::Slot16],
            )
            .build();

        let pcr_selection_list_size = pcr_selection_list.len();
        let actual: TPML_PCR_SELECTION = pcr_selection_list.into();
        assert_eq!(actual.count as usize, pcr_selection_list_size);
        // No size has been chosen so the default will be set.
        assert_eq!(
            actual.pcrSelections[0].sizeofSelect,
            PcrSelectSize::default().to_u8().unwrap()
        );
        assert_eq!(
            actual.pcrSelections[0].hash,
            Into::<TPM2_ALG_ID>::into(HashingAlgorithm::Sha256)
        );
        assert_eq!(actual.pcrSelections[0].pcrSelect[0], 0b0000_0001);
        assert_eq!(actual.pcrSelections[0].pcrSelect[1], 0b0000_0001);
        assert_eq!(actual.pcrSelections[0].pcrSelect[2], 0b0000_0001);
    }

    #[test]
    fn test_multiple_selection() {
        let pcr_selection_list = PcrSelectionListBuilder::new()
            .with_selection(
                HashingAlgorithm::Sha256,
                &[PcrSlot::Slot0, PcrSlot::Slot8, PcrSlot::Slot16],
            )
            .with_selection(
                HashingAlgorithm::Sha1,
                &[
                    PcrSlot::Slot1,
                    PcrSlot::Slot8,
                    PcrSlot::Slot10,
                    PcrSlot::Slot20,
                    PcrSlot::Slot23,
                ],
            )
            .build();

        let pcr_selection_list_size = pcr_selection_list.len();
        let actual: TPML_PCR_SELECTION = pcr_selection_list.into();
        assert_eq!(actual.count as usize, pcr_selection_list_size);
        for pcr_selection in actual.pcrSelections[..actual.count as usize].iter() {
            assert_eq!(
                pcr_selection.sizeofSelect,
                PcrSelectSize::default().to_u8().unwrap()
            );
            // The order is not specified.
            match HashingAlgorithm::try_from(pcr_selection.hash).unwrap() {
                HashingAlgorithm::Sha256 => {
                    assert_eq!(pcr_selection.pcrSelect[0], 0b0000_0001);
                    assert_eq!(pcr_selection.pcrSelect[1], 0b0000_0001);
                    assert_eq!(pcr_selection.pcrSelect[2], 0b0000_0001);
                }
                HashingAlgorithm::Sha1 => {
                    assert_eq!(pcr_selection.pcrSelect[0], 0b0000_0010);
                    assert_eq!(pcr_selection.pcrSelect[1], 0b0000_0101);
                    assert_eq!(pcr_selection.pcrSelect[2], 0b1001_0000);
                }
                _ => panic!("Encountered incorrect Hashing Algorithm"),
            }
        }
    }

    #[test]
    fn test_multiple_conversions() {
        let pcr_selection_list = PcrSelectionListBuilder::new()
            .with_size_of_select(Default::default())
            .with_selection(
                HashingAlgorithm::Sha256,
                &[PcrSlot::Slot0, PcrSlot::Slot8, PcrSlot::Slot16],
            )
            .build();
        let pcr_selection_list_size = pcr_selection_list.len();
        let converted: TPML_PCR_SELECTION = pcr_selection_list.into();
        assert_eq!(converted.count as usize, pcr_selection_list_size);

        let from_converted = PcrSelectionList::try_from(converted).unwrap();
        let re_converted: TPML_PCR_SELECTION = from_converted.into();

        assert_eq!(converted.count, re_converted.count);
        assert_eq!(
            converted.pcrSelections[0].sizeofSelect,
            re_converted.pcrSelections[0].sizeofSelect
        );
        assert_eq!(
            converted.pcrSelections[0].hash,
            re_converted.pcrSelections[0].hash
        );
        assert_eq!(
            converted.pcrSelections[0].pcrSelect[0],
            re_converted.pcrSelections[0].pcrSelect[0]
        );
        assert_eq!(
            converted.pcrSelections[0].pcrSelect[1],
            re_converted.pcrSelections[0].pcrSelect[1]
        );
        assert_eq!(
            converted.pcrSelections[0].pcrSelect[2],
            re_converted.pcrSelections[0].pcrSelect[2]
        );
    }

    #[test]
    fn test_conversion_of_data_with_invalid_pcr_select_bit_flags() {
        let mut tpml_pcr_selection: TPML_PCR_SELECTION = PcrSelectionListBuilder::new()
            .with_selection(
                HashingAlgorithm::Sha256,
                &[PcrSlot::Slot0, PcrSlot::Slot8, PcrSlot::Slot16],
            )
            .build()
            .into();

        // Size of select is 3 the maximum value supported.
        // Setting a value in the fourth octet is creating
        // a none supported bit flag
        tpml_pcr_selection.pcrSelections[0].pcrSelect[3] = 1;

        // The try_from should then fail.
        PcrSelectionList::try_from(tpml_pcr_selection).unwrap_err();
    }

    #[test]
    fn test_conversion_of_data_with_invalid_size_of_select() {
        let mut tpml_pcr_selection: TPML_PCR_SELECTION = PcrSelectionListBuilder::new()
            .with_selection(
                HashingAlgorithm::Sha256,
                &[PcrSlot::Slot0, PcrSlot::Slot8, PcrSlot::Slot16],
            )
            .build()
            .into();

        // 1,2,3,4 are theonly valid values for sizeofSelect.
        tpml_pcr_selection.pcrSelections[0].sizeofSelect = 20;

        // The try_from should then fail.
        PcrSelectionList::try_from(tpml_pcr_selection).unwrap_err();
    }

    #[test]
    fn test_conversion_of_data_with_invalid_hash_alg_id() {
        let mut tpml_pcr_selection: TPML_PCR_SELECTION = PcrSelectionListBuilder::new()
            .with_selection(
                HashingAlgorithm::Sha256,
                &[PcrSlot::Slot0, PcrSlot::Slot8, PcrSlot::Slot16],
            )
            .build()
            .into();

        // Si
        tpml_pcr_selection.pcrSelections[0].hash = TPM2_ALG_LAST + 1;

        // The try_from should then fail.
        PcrSelectionList::try_from(tpml_pcr_selection).unwrap_err();
    }
}
