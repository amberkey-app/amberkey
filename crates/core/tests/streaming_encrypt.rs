//! Streaming encryption must produce an ordinary age file. If it did not, a
//! family would hold ciphertext that `age -d` and the recovery tool refuse,
//! which is worse than the size limit it replaces.

use amberkey_core::bundle::{decrypt_file, encrypt_file, StreamingEncryptor};

fn identity() -> (String, String) {
    use age::secrecy::ExposeSecret;
    let id = amberkey_core::bundle::generate_identity();
    let secret = id.to_string().expose_secret().to_string();
    let recipient = amberkey_core::bundle::recipient_from_secret(&secret).unwrap();
    (secret, recipient)
}

fn stream_encrypt(recipient: &str, data: &[u8], chunk: usize) -> Vec<u8> {
    let mut enc = StreamingEncryptor::new(recipient).unwrap();
    let mut out = Vec::new();
    for part in data.chunks(chunk) {
        out.extend_from_slice(&enc.update(part).unwrap());
    }
    out.extend_from_slice(&enc.finish().unwrap());
    out
}

#[test]
fn streamed_output_decrypts_to_the_original() {
    let (secret, recipient) = identity();
    // Deliberately not a multiple of the chunk size, and larger than age's
    // internal 64 KiB payload chunking so the streaming path is really used.
    let data: Vec<u8> = (0..300_000u32).map(|i| (i % 251) as u8).collect();
    let ct = stream_encrypt(&recipient, &data, 7_000);
    let back = decrypt_file(&secret, &ct).unwrap();
    assert_eq!(back, data, "streamed ciphertext must round-trip");
}

#[test]
fn chunk_size_does_not_change_the_plaintext() {
    let (secret, recipient) = identity();
    let data: Vec<u8> = (0..200_000u32).map(|i| (i % 97) as u8).collect();
    for chunk in [1, 1024, 65_536, 300_000] {
        let ct = stream_encrypt(&recipient, &data, chunk);
        assert_eq!(decrypt_file(&secret, &ct).unwrap(), data, "chunk size {chunk} broke it");
    }
}

#[test]
fn a_streamed_file_is_readable_by_the_one_shot_path_and_vice_versa() {
    let (secret, recipient) = identity();
    let data = b"the recovery tool must open either one".to_vec();
    let streamed = stream_encrypt(&recipient, &data, 8);
    let oneshot = encrypt_file(&recipient, &data).unwrap();
    assert_eq!(decrypt_file(&secret, &streamed).unwrap(), data);
    assert_eq!(decrypt_file(&secret, &oneshot).unwrap(), data);
}

#[test]
fn empty_input_still_produces_a_valid_file() {
    let (secret, recipient) = identity();
    let ct = stream_encrypt(&recipient, &[], 1024);
    assert_eq!(decrypt_file(&secret, &ct).unwrap(), Vec::<u8>::new());
}

#[test]
fn using_it_after_finish_is_an_error_not_a_corrupt_file() {
    let (_secret, recipient) = identity();
    let mut enc = StreamingEncryptor::new(&recipient).unwrap();
    enc.update(b"hello").unwrap();
    enc.finish().unwrap();
    assert!(enc.update(b"more").is_err(), "must not silently append after finish");
    assert!(enc.finish().is_err(), "must not finish twice");
}

// ---- streaming decrypt ----

use amberkey_core::bundle::StreamingDecryptor;

fn stream_decrypt(secret: &str, ct: Vec<u8>, chunk: usize) -> Vec<u8> {
    let mut dec = StreamingDecryptor::new(secret, ct).unwrap();
    let mut out = Vec::new();
    loop {
        let part = dec.read_chunk(chunk).unwrap();
        if part.is_empty() {
            break;
        }
        out.extend_from_slice(&part);
    }
    out
}

#[test]
fn streaming_decrypt_round_trips_a_streamed_file() {
    let (secret, recipient) = identity();
    let data: Vec<u8> = (0..250_000u32).map(|i| (i % 233) as u8).collect();
    let ct = stream_encrypt(&recipient, &data, 4_096);
    assert_eq!(stream_decrypt(&secret, ct, 16_384), data);
}

#[test]
fn streaming_decrypt_reads_a_one_shot_file_too() {
    // Files already written by encrypt_file must keep opening.
    let (secret, recipient) = identity();
    let data = b"an archive encrypted before streaming existed".to_vec();
    let ct = encrypt_file(&recipient, &data).unwrap();
    assert_eq!(stream_decrypt(&secret, ct, 8), data);
}

#[test]
fn decrypt_chunk_size_does_not_change_the_plaintext() {
    let (secret, recipient) = identity();
    let data: Vec<u8> = (0..120_000u32).map(|i| (i % 61) as u8).collect();
    let ct = stream_encrypt(&recipient, &data, 9_999);
    for chunk in [1, 512, 65_536, 1_000_000] {
        assert_eq!(stream_decrypt(&secret, ct.clone(), chunk), data, "chunk {chunk}");
    }
}

#[test]
fn the_wrong_key_fails_rather_than_returning_garbage() {
    let (_secret, recipient) = identity();
    let (other_secret, _) = identity();
    let ct = stream_encrypt(&recipient, b"secret things", 64);
    assert!(StreamingDecryptor::new(&other_secret, ct).is_err(), "must not decrypt with a foreign key");
}
