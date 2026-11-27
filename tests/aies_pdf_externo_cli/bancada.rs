use std::io::Read;

pub(super) fn pedido(canal: &mut impl Read) -> (Vec<u8>, String, String) {
    let mut cabecalho = Vec::new();
    let mut byte = [0];
    while !cabecalho.ends_with(b"\r\n\r\n") {
        assert!(cabecalho.len() < 16_384);
        canal.read_exact(&mut byte).unwrap();
        cabecalho.push(byte[0]);
    }
    let texto = std::str::from_utf8(&cabecalho).unwrap();
    let inicio = texto.lines().next().unwrap().to_owned();
    let tamanho = texto
        .lines()
        .find_map(|linha| {
            let (nome, valor) = linha.split_once(':')?;
            nome.eq_ignore_ascii_case("content-length")
                .then(|| valor.trim().parse::<usize>().unwrap())
        })
        .unwrap_or(0);
    assert!(tamanho < 3 * 1024 * 1024, "limite da bancada sintética");
    let mut corpo = vec![0; tamanho];
    canal.read_exact(&mut corpo).unwrap();
    cabecalho.extend(corpo);
    let acao = inicio
        .split_whitespace()
        .nth(1)
        .unwrap()
        .split_once('?')
        .and_then(|(_, query)| query.split('&').find_map(|par| par.strip_prefix("acao=")))
        .unwrap_or("desconhecida")
        .to_owned();
    let metodo = inicio.split_whitespace().next().unwrap().to_owned();
    (cabecalho, metodo, acao)
}
