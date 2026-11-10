//! Transições temporais do BDD-60-05, apenas no SQLite efêmero.
//! Nenhuma transição é atribuída à instalação real sem cassette.
use super::*;
use iesei::acesso_sessao;
use iesei::acesso_versao::{self, AcessoVersao};

/// Executa no SQLite uma transição local antes de responder ao pedido HTTP.
/// Nenhum estado é alterado sem que a requisição esperada seja observada.
pub(super) fn aplicar_transicao(
    modo: EstadoExtra,
    tcp: &TcpStream,
    banco: &Connection,
    instalacao: &Instalacao,
) -> bool {
    match modo {
        EstadoExtra::RevogadoAoVisualizar => revogar_antes_de_servir_corpo(tcp, banco, instalacao),
        EstadoExtra::TituloTrocadoAoVisualizar => {
            trocar_titulo_antes_de_servir_corpo(tcp, banco, instalacao)
        }
        EstadoExtra::SessaoRevogadaAoVisualizar => {
            revogar_sessao_antes_de_servir_corpo(tcp, banco, instalacao)
        }
        EstadoExtra::SessaoRevogadaAntesDaLista => {
            revogar_sessao_antes_de_servir_lista(tcp, banco, instalacao)
        }
        EstadoExtra::VersaoRevogadaAoVisualizar => {
            revogar_versao_antes_de_servir_corpo(tcp, banco, instalacao)
        }
        EstadoExtra::CanceladoAntesDaLista => {
            cancelar_antes_de_servir_lista(tcp, banco, instalacao)
        }
        _ => false,
    }
}

/// Espia apenas a primeira linha sem consumir a requisição HTTP original.
fn eh_get_de_conteudo(tcp: &TcpStream) -> bool {
    let mut inicio = [0_u8; 1024];
    let tamanho = tcp.peek(&mut inicio).unwrap();
    let linha = String::from_utf8_lossy(&inicio[..tamanho]);
    linha.starts_with("GET ") && linha.contains("acao=documento_visualizar")
}

fn eh_get_de_lista(tcp: &TcpStream) -> bool {
    let mut inicio = [0_u8; 1024];
    let tamanho = tcp.peek(&mut inicio).unwrap();
    let linha = String::from_utf8_lossy(&inicio[..tamanho]);
    linha.starts_with("GET ") && linha.contains("acao=documento_versao_listar")
}

/// A árvore já foi publicada; a origem altera o título antes do GET do
/// conteúdo. Hipótese de corrida sintética, não transição capturada na `UnB`.
pub(super) fn trocar_titulo_antes_de_servir_corpo(
    tcp: &TcpStream,
    banco: &Connection,
    instalacao: &Instalacao,
) -> bool {
    if !eh_get_de_conteudo(tcp) {
        return false;
    }
    let alterados = banco
        .execute(
            "UPDATE documento_simulado SET numero=?1
             WHERE origem=?2 AND cenario=?3 AND id_documento=?4 AND numero=?5",
            rusqlite::params![DOC_B, instalacao.url_base(), CENARIO, "42", DOC_A],
        )
        .unwrap();
    assert_eq!(alterados, 1, "a hipótese altera a identidade do documento");
    true
}

/// Simula a revogação do href já publicado entre a árvore e a resposta
/// documental. O peek não consome bytes: o servidor recebe o HTTP original.
pub(super) fn revogar_antes_de_servir_corpo(
    tcp: &TcpStream,
    banco: &Connection,
    instalacao: &Instalacao,
) -> bool {
    if !eh_get_de_conteudo(tcp) {
        return false;
    }
    assert_eq!(
        cancelamento_documento::aplicar(
            banco,
            instalacao,
            CENARIO,
            "42",
            "7",
            "Revogado entre leituras"
        )
        .unwrap(),
        Aplicacao::Aplicada
    );
    true
}

/// Revogação global da sessão depois de publicada a árvore, sem alterar DOC.
pub(super) fn revogar_sessao_antes_de_servir_corpo(
    tcp: &TcpStream,
    banco: &Connection,
    instalacao: &Instalacao,
) -> bool {
    if !eh_get_de_conteudo(tcp) {
        return false;
    }
    acesso_sessao::bloquear(banco, instalacao, CENARIO).unwrap();
    true
}

/// A sessão muda para recusada depois da árvore e antes do histórico.
/// O SQLite retém os documentos; apenas a autorização é alterada.
pub(super) fn revogar_sessao_antes_de_servir_lista(
    tcp: &TcpStream,
    banco: &Connection,
    instalacao: &Instalacao,
) -> bool {
    if !eh_get_de_lista(tcp) {
        return false;
    }
    acesso_sessao::bloquear(banco, instalacao, CENARIO).unwrap();
    true
}

/// Revoga somente a versão 1 após a publicação da lista de versões.
/// A revisão 2 permanece disponível. Esta corrida é hipótese sintética.
pub(super) fn revogar_versao_antes_de_servir_corpo(
    tcp: &TcpStream,
    banco: &Connection,
    instalacao: &Instalacao,
) -> bool {
    if !eh_get_de_conteudo(tcp) {
        return false;
    }
    let mut inicio = [0_u8; 1024];
    let tamanho = tcp.peek(&mut inicio).unwrap();
    if !String::from_utf8_lossy(&inicio[..tamanho]).contains("versao=1") {
        return false;
    }
    acesso_versao::restringir(
        banco,
        instalacao,
        CENARIO,
        "42",
        NonZeroU32::new(1).unwrap(),
    )
    .unwrap();
    true
}

/// Um histórico já anunciado na árvore pode deixar de ser acessível quando
/// o documento é cancelado. A corrida é sintética, não capturada na `UnB`.
pub(super) fn cancelar_antes_de_servir_lista(
    tcp: &TcpStream,
    banco: &Connection,
    instalacao: &Instalacao,
) -> bool {
    if !eh_get_de_lista(tcp) {
        return false;
    }
    assert_eq!(
        cancelamento_documento::aplicar(
            banco,
            instalacao,
            CENARIO,
            "42",
            "7",
            "Cancelamento entre árvore e histórico"
        )
        .unwrap(),
        Aplicacao::Aplicada
    );
    true
}

/// A recusa e a leitura da CLI não alteram os dois documentos independentes.
fn conferir_duplicidade_entre_processos(db: &Connection, instalacao: &Instalacao) {
    for (id, nup, conteudo) in [
        ("42", NUP_A, "primeiro processo"),
        ("43", NUP_B, "segundo processo"),
    ] {
        let doc = documento_versoes::ler(db, instalacao, CENARIO, id)
            .unwrap()
            .expect("documento preservado no SQLite");
        assert_eq!(doc.numero, DOC_A);
        assert_eq!(doc.numero_processo, nup);
        assert!(
            doc.revisoes[0].html_corpo.contains(conteudo),
            "o corpo do documento não pode ser trocado por outro processo"
        );
    }
}

/// Confere diretamente no SQLite o efeito da transição local ensaiada.
pub(super) fn conferir_estado_apos_cli(
    modo: EstadoExtra,
    transicao: bool,
    db: &Connection,
    instalacao: &Instalacao,
) {
    if matches!(modo, EstadoExtra::RevogadoAoVisualizar) {
        assert!(transicao, "CLI não tentou seguir a URL antes oferecida");
        assert!(matches!(
            cancelamento_documento::consultar(db, instalacao, CENARIO, "42", "7"),
            Ok(Some(cancelamento_documento::Situacao::Cancelado { .. }))
        ));
    }
    if matches!(modo, EstadoExtra::CanceladoAntesDaLista) {
        assert!(
            transicao,
            "a CLI deveria requisitar o histórico já oferecido"
        );
        assert!(matches!(
            cancelamento_documento::consultar(db, instalacao, CENARIO, "42", "7"),
            Ok(Some(cancelamento_documento::Situacao::Cancelado { .. }))
        ));
        assert_eq!(
            cancelamento_documento::consultar(db, instalacao, CENARIO, "43", "8").unwrap(),
            Some(cancelamento_documento::Situacao::Disponivel),
            "o cancelamento não afeta o documento de outro processo"
        );
    }
    if matches!(
        modo,
        EstadoExtra::SessaoRevogadaAoVisualizar | EstadoExtra::SessaoRevogadaAntesDaLista
    ) {
        assert!(transicao, "CLI não seguiu a ação oferecida na árvore");
        assert_eq!(
            acesso_sessao::consultar(db, instalacao, CENARIO).unwrap(),
            acesso_sessao::AcessoSessao::Recusado
        );
        assert_eq!(
            cancelamento_documento::consultar(db, instalacao, CENARIO, "42", "7").unwrap(),
            Some(cancelamento_documento::Situacao::Disponivel),
            "negação global de sessão não cancela o documento"
        );
    }
    if matches!(modo, EstadoExtra::VersaoRevogadaAoVisualizar) {
        assert!(transicao, "CLI não buscou a versão 1 após o histórico");
        assert_eq!(
            acesso_versao::consultar(db, instalacao, CENARIO, "42", NonZeroU32::new(1).unwrap())
                .unwrap(),
            AcessoVersao::Recusado
        );
        assert_eq!(
            acesso_versao::consultar(db, instalacao, CENARIO, "42", NonZeroU32::new(2).unwrap())
                .unwrap(),
            AcessoVersao::Acessivel,
            "a revogação da versão 1 não afeta a revisão 2"
        );
    }
    if matches!(modo, EstadoExtra::NumeroIgualEmOutroProcesso) {
        conferir_duplicidade_entre_processos(db, instalacao);
    }
    if matches!(modo, EstadoExtra::PdfNumeroDuplicado) {
        let pdfs = iesei::pdf_externo::documentos(db, instalacao, CENARIO, "7").unwrap();
        assert_eq!(pdfs.len(), 1, "o PDF continua cadastrado no SQLite");
        assert_eq!(pdfs[0].numero_sei, DOC_A);
        assert_eq!(
            iesei::pdf_externo::conteudo_do_anexo(db, instalacao, CENARIO, &pdfs[0].id.to_string())
                .unwrap(),
            Some(b"%PDF-1.4\n%%EOF\n".to_vec()),
            "a recusa da CLI não altera os bytes do anexo"
        );
        assert_eq!(
            documento_versoes::ler(db, instalacao, CENARIO, "42")
                .unwrap()
                .unwrap()
                .numero,
            DOC_A,
            "o documento interno permanece íntegro"
        );
    }
    if matches!(modo, EstadoExtra::TituloTrocadoAoVisualizar) {
        assert!(transicao, "CLI não buscou o corpo depois de ler a árvore");
        let observado: String = db
            .query_row(
                "SELECT numero FROM documento_simulado
             WHERE origem=?1 AND cenario=?2 AND id_documento='42'",
                rusqlite::params![instalacao.url_base(), CENARIO],
                |linha| linha.get(0),
            )
            .unwrap();
        assert_eq!(observado, DOC_B, "a transição persiste no SQLite");
        assert_eq!(
            documento_versoes::ler(db, instalacao, CENARIO, "43")
                .unwrap()
                .unwrap()
                .numero,
            DOC_B,
            "o segundo documento não foi alterado"
        );
    }
}
