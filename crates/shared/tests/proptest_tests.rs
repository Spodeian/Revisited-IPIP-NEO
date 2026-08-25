use proptest::prelude::*;
use shared::export::{
    decode_responses_from_url_code, encode_responses_to_url_code, pack_3bit_stream,
    unpack_3bit_stream,
};
use shared::questionnaire::{QuestionnaireState, Response};

fn random_response_strategy() -> impl Strategy<Value = Option<Response>> {
    prop_oneof![
        Just(None),
        Just(Some(Response::StronglyDisagree)),
        Just(Some(Response::Disagree)),
        Just(Some(Response::Neutral)),
        Just(Some(Response::Agree)),
        Just(Some(Response::StronglyAgree)),
    ]
}

proptest! {
    #[test]
    fn test_3bit_stream_packing_roundtrip(raw_vals in prop::collection::vec(0u8..=5u8, 1..300)) {
        let packed = pack_3bit_stream(&raw_vals);
        let unpacked = unpack_3bit_stream(&packed, raw_vals.len());
        prop_assert_eq!(raw_vals, unpacked);
    }

    #[test]
    fn test_url_code_lossless_roundtrip(responses in prop::collection::vec(random_response_strategy(), 221)) {
        let mut state = QuestionnaireState::from_embedded_data();
        let mut has_answered = false;

        for (i, &resp) in responses.iter().enumerate() {
            if i < state.questions.len() {
                state.questions[i].response = resp;
                if resp.is_some() {
                    has_answered = true;
                }
            }
        }
        state.rebuild_cache();

        let encoded = encode_responses_to_url_code(&state);
        prop_assert!(encoded.len() <= 120, "Encoded string length {} exceeded expected bounds", encoded.len());

        let mut restored = QuestionnaireState::from_embedded_data();
        let decode_res = decode_responses_from_url_code(&mut restored, &encoded);

        if has_answered {
            prop_assert!(decode_res.is_ok());
            for i in 0..state.questions.len() {
                prop_assert_eq!(
                    state.questions[i].response,
                    restored.questions[i].response,
                    "Mismatch at question index {}",
                    i
                );
            }
            // Accumulator values must match exactly
            for (&k, acc1) in &state.meta_trait_acc {
                let acc2 = restored.meta_trait_acc.get(&k).unwrap();
                prop_assert_eq!(acc1.raw_score, acc2.raw_score);
                prop_assert_eq!(acc1.total_abs_weight, acc2.total_abs_weight);
                prop_assert_eq!(acc1.total_sq_weight, acc2.total_sq_weight);
            }
        }
    }

    #[test]
    fn test_undo_stack_invariants(steps in prop::collection::vec((0usize..221, random_response_strategy()), 1..30)) {
        let mut state = QuestionnaireState::from_embedded_data();
        let initial_state = state.clone();

        for &(idx, resp) in &steps {
            if let Some(r) = resp {
                state.answer_question(idx, r);
            } else {
                state.clear_response(idx);
            }
        }

        // Now undo all steps
        for _ in 0..steps.len() {
            let _ = state.undo();
        }

        for i in 0..state.questions.len() {
            prop_assert_eq!(state.questions[i].response, initial_state.questions[i].response);
        }
    }
}
