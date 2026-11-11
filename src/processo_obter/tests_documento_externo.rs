//! RED/GREEN de `ies documento DOC` externo (Refs #60, fatia 3): o
//! `.src` `documento_download_anexo` da árvore vira o quinto pedido, um
//! GET binário do link publicado (nunca montado), contado e lido pelo
//! `anexo_documento`. Telas SINTÉTICAS com a forma das cassettes
//! `documento-numero-e-busca-*`; anexo com os cabeçalhos observados.
use super::tests_apoio::*;
use super::*;
use crate::acao_do_sei::AcaoDoSei;
use crate::anexo_documento::{Disposicao, ErroAnexo, LimiteDoAnexo};
use crate::arvore_processo::tests::{NUP, arvore, no_documento, no_processo};
use crate::arvore_processo::tests_conteudo::{
    HASH_CONTEUDO, com_src, src_anexo, src_da_arvore_do_documento,
};
use crate::arvore_processo::tests_tela::tela_do_processo;
use crate::documento::NumeroDocumento;
use crate::formulario_web::{RespostaDoPost, SubmissaoDerivada};
use crate::transporte::RespostaWeb;
use crate::transporte::resposta_binaria::RespostaBinaria;
use crate::transporte::sessao::CookieDaSessao;
use std::io::Cursor;

const DOC: &str = "0000101";
const ID_DOC: &str = "21";

/// O roteiro do HTML e a única resposta binária do anexo.
struct ComAnexo {
    roteiro: Roteiro,
    anexo: Option<RespostaBinaria<Cursor<Vec<u8>>>>,
    limite_pedido: Option<usize>,
}

impl EnvioDoControle for ComAnexo {
    type Corpo = Cursor<Vec<u8>>;
    fn post(
        &mut self,
        submissao: SubmissaoDerivada,
        cookie: &str,
    ) -> Result<RespostaDoPost<Self::Corpo>, FalhaTransporte> {
        self.roteiro.post(submissao, cookie)
    }
    fn get(
        &mut self,
        url: &str,
        cookie: &str,
    ) -> Result<RespostaWeb<Self::Corpo>, FalhaTransporte> {
        EnvioDoControle::get(&mut self.roteiro, url, cookie)
    }
}

impl EnvioDoProcesso for ComAnexo {
    fn get_sem_seguir(
        &mut self,
        url: &str,
        cookie: &str,
    ) -> Result<RespostaDoPost<Self::Corpo>, FalhaTransporte> {
        self.roteiro.get_sem_seguir(url, cookie)
    }
    fn get_binario(
        &mut self,
        url: &str,
        cookie: &str,
        limite: usize,
    ) -> Result<RespostaBinaria<Self::Corpo>, FalhaTransporte> {
        assert_eq!(cookie, COOKIE);
        self.roteiro.pedidos.push(get(url));
        self.limite_pedido = Some(limite);
        Ok(self.anexo.take().expect("um único GET do anexo"))
    }
}

fn binaria(
    status: u16,
    cabecalhos: &[(&str, &str)],
    corpo: &[u8],
) -> RespostaBinaria<Cursor<Vec<u8>>> {
    RespostaBinaria {
        status,
        cabecalhos: cabecalhos
            .iter()
            .map(|(n, v)| ((*n).to_owned(), (*v).to_owned()))
            .collect(),
        corpo: Cursor::new(corpo.to_vec()),
    }
}

fn pdf() -> RespostaBinaria<Cursor<Vec<u8>>> {
    binaria(
        200,
        &[
            ("content-type", "application/pdf"),
            ("content-disposition", "inline; filename=\"Minuta.pdf\""),
            ("content-length", "12"),
        ],
        b"%PDF-1.7 abc",
    )
}

/// Controle, POST, tela e árvore com o `.src` do anexo no nó do documento.
fn ate_a_arvore() -> Vec<Resposta> {
    let tela = tela_do_processo(UNIDADE, &[src_da_arvore_do_documento(ID, ID_DOC)])
        .replace("</body>", &format!("{}</body>", formulario_rapido()));
    let arvore = com_src(
        &arvore(&[no_processo(0, ID, NUP), no_documento(1, ID_DOC, ID, DOC)]),
        1,
        &src_anexo(),
    )
    .replace("svg/documento_interno.svg", "svg/documento_pdf.svg");
    vec![
        ok(controle(&[("7", "OUTRO-1")])),
        r302(&trabalhar_rapida_documento(ID_DOC)),
        ok(tela),
        ok(arvore),
    ]
}

fn rodar(
    amb: &Ambiente,
    anexo: RespostaBinaria<Cursor<Vec<u8>>>,
    pedido_do_anexo: PedidoDoAnexo<'_>,
) -> (Result<DocumentoObtido, ErroDocumentoObter>, ComAnexo) {
    rodar_com(amb, ate_a_arvore(), anexo, pedido_do_anexo)
}

fn rodar_com(
    amb: &Ambiente,
    respostas: Vec<Resposta>,
    anexo: RespostaBinaria<Cursor<Vec<u8>>>,
    pedido_do_anexo: PedidoDoAnexo<'_>,
) -> (Result<DocumentoObtido, ErroDocumentoObter>, ComAnexo) {
    let url = url_controle();
    let sessao = SessaoDoProcesso {
        url_do_controle: &url,
        origem: ORIGEM,
        cookie: &CookieDaSessao::ler(&amb.cookie),
    };
    let numero = NumeroDocumento::novo(DOC).unwrap();
    let pedido = PedidoDoDocumento {
        numero: &numero,
        unidade: None,
        processo: None,
        anexo: pedido_do_anexo,
    };
    let mut envio = ComAnexo {
        roteiro: Roteiro::novo(respostas),
        anexo: Some(anexo),
        limite_pedido: None,
    };
    let r = obter_documento(&sessao, &pedido, &amb.indice, &mut envio);
    assert_eq!(envio.roteiro.respostas_restantes(), 0);
    (r, envio)
}

#[test]
fn pdf_externo_e_o_quinto_pedido_pelo_link_da_arvore() {
    let amb = ambiente();
    let (r, envio) = rodar(&amb, pdf(), PedidoDoAnexo::default());
    let d = r.unwrap();
    assert_eq!(
        envio.roteiro.pedidos,
        [
            get(&url_controle()),
            post(&absoluta(&action_rapida()), "txtPesquisaRapida=0000101"),
            get(&absoluta(&trabalhar_rapida_documento(ID_DOC))),
            get(&absoluta(&src_da_arvore_do_documento(ID, ID_DOC))),
            get(&absoluta(&src_anexo())),
        ]
    );
    assert_eq!(envio.limite_pedido, Some(64 * 1024 * 1024));
    assert_eq!(d.requisicoes, 5);
    assert_eq!(d.numero, DOC);
    assert_eq!(d.tipo, "");
    assert_eq!(d.texto, "");
    assert_eq!(d.natureza.icone, "documento_pdf");
    let anexo = d.anexo.as_ref().expect("anexo");
    assert_eq!(anexo.mime, "application/pdf");
    assert_eq!(anexo.tamanho, 12);
    assert_eq!(anexo.nome_original, "Minuta.pdf");
    assert_eq!(anexo.disposicao, Disposicao::Inline);
    assert_eq!(anexo.texto(), None);
    assert!(!format!("{d:?}").contains(HASH_CONTEUDO));
    // #88: o arquivo não serve rodapé; sem `--versao`, a barra não é lida.
    assert_eq!(d.versoes.lida, None);
    assert_eq!(d.versoes.atual, Err(SemVersaoAtual::NaoServidaNoConteudo));
    assert_eq!(d.versoes.foto, None);
}

#[test]
fn dado_externo_sem_src_com_anexo_no_cadastro_quando_obtenho_entao_baixo_original() {
    // Dado externo da forma Todos tests/paginas_reais/documento_alterar_recebido.html
    // (@main 07/11/2026): a arvore nao publica .src, mas a barra publica
    // documento_alterar_recebido e o cadastro publica download do anexo.
    // Quando executo ies documento DOC --json, entao sigo os dois links
    // publicados, obtenho o PDF e conto seis requisicoes, sem POST extra.
    use crate::arvore_processo::tests_barra_documento::{barra, com_barra};
    let amb = ambiente();
    let arvore_sem_src = arvore(&[no_processo(0, ID, NUP), no_documento(1, ID_DOC, ID, DOC)])
        .replace("svg/documento_interno.svg", "svg/documento_pdf.svg");
    let barra = barra(ID, ID_DOC, None)
        .replace("acao=documento_alterar", "acao=documento_alterar_recebido");
    let arvore = com_barra(&arvore_sem_src, 1, &barra);
    let mut respostas = ate_a_arvore();
    respostas[3] = ok(arvore);
    let cadastro = format!(
        r#"<script>
          objTabelaAnexos.adicionarAcoes('901','<a href="{}" target="_blank">Baixar</a>');
          </script><form id="frmDocumentoCadastro">
          <input id="hdnIdDocumento" value="21">
          <input id="hdnIdProcedimento" value="10">
          <label id="lblSerieTitulo"></label>
          <select id="selSerie"><option value="293" selected>Anexo</option></select>
          <input id="txtDescricao" value="PDF de teste">
          <input id="txtNomeArvore" value="Anexo externo">
          <textarea id="txaObservacoes"></textarea>
          <input name="rdoNivelAcesso" value="0" checked>
          <input id="hdnAnexos" value="901±Minuta.pdf±data±12">
          </form>"#,
        src_anexo()
    );
    respostas.push(ok(cadastro));
    let (r, envio) = rodar_com(&amb, respostas, pdf(), PedidoDoAnexo::default());
    let d = r.unwrap();
    assert_eq!(d.requisicoes, 6);
    assert_eq!(envio.roteiro.pedidos.len(), 6);
    assert_eq!(envio.roteiro.pedidos[5], get(&absoluta(&src_anexo())));
    assert_eq!(d.anexo.unwrap().mime, "application/pdf");
}

#[test]
fn externo_sem_src_com_saida_preserva_original_e_recusa_copia_divergente() {
    // Apoio técnico ao BDD-60-03: --saida DIR grava bytes do anexo cadastrado.
    use crate::arvore_processo::tests_barra_documento::{barra, com_barra};
    let dir =
        std::env::temp_dir().join(format!("ies-externo-cadastro-saida-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let respostas = || {
        let arvore_sem_src = arvore(&[no_processo(0, ID, NUP), no_documento(1, ID_DOC, ID, DOC)])
            .replace("svg/documento_interno.svg", "svg/documento_pdf.svg");
        let barra = barra(ID, ID_DOC, None)
            .replace("acao=documento_alterar", "acao=documento_alterar_recebido");
        let mut respostas = ate_a_arvore();
        respostas[3] = ok(com_barra(&arvore_sem_src, 1, &barra));
        respostas.push(ok(format!(
            r#"<script>objTabelaAnexos.adicionarAcoes('901','<a href="{}">Baixar</a>');</script>
            <form id="frmDocumentoCadastro">
            <input id="hdnIdDocumento" value="21">
            <input id="hdnIdProcedimento" value="10">
            <label id="lblSerieTitulo"></label>
            <select id="selSerie"><option selected>Anexo</option></select>
            <input id="txtDescricao" value="">
            <input id="txtNomeArvore" value="Externo">
            <textarea id="txaObservacoes"></textarea>
            <input name="rdoNivelAcesso" value="0" checked>
            <input id="hdnAnexos" value="901±Minuta.pdf±data±12">
            </form>"#,
            src_anexo()
        )));
        respostas
    };
    let pedido = PedidoDoAnexo {
        cache_host: None,
        limite: LimiteDoAnexo::default(),
        saida: Some(&dir),
    };
    let amb = ambiente();
    let (lido, envio) = rodar_com(&amb, respostas(), pdf(), pedido);
    let doc = lido.unwrap();
    assert_eq!(doc.requisicoes, 6);
    assert_eq!(envio.roteiro.pedidos.len(), 6);
    let mut anexo = doc.anexo.unwrap();
    let arquivo = anexo.tirar_arquivo().unwrap();
    assert_eq!(arquivo.nome(), "0000101.pdf");
    arquivo.concluir().unwrap();
    assert_eq!(
        std::fs::read(dir.join("0000101.pdf")).unwrap(),
        b"%PDF-1.7 abc"
    );

    // Cópia divergente não pode ser silenciosamente substituída.
    std::fs::write(dir.join("0000101.pdf"), b"edicao local").unwrap();
    let amb = ambiente();
    let (recusa, _) = rodar_com(&amb, respostas(), pdf(), pedido);
    assert_eq!(
        recusa.unwrap_err(),
        ErroDocumentoObter::Anexo(ErroAnexo::ArquivoJaExiste)
    );
    assert_eq!(
        std::fs::read(dir.join("0000101.pdf")).unwrap(),
        b"edicao local"
    );
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn html_recebido_publica_o_texto_visivel() {
    let amb = ambiente();
    let html = b"<html><head><title>SEI/ORIGEM - 0403231 - Despacho</title></head>\
        <body><p>Despacho do \xf3rg\xe3o de origem.</p></body></html>";
    let (r, _) = rodar(
        &amb,
        binaria(
            200,
            &[
                ("content-type", "text/html;;charset=iso-8859-1"),
                ("content-disposition", "inline; filename=\"despacho.html\""),
            ],
            html,
        ),
        PedidoDoAnexo::default(),
    );
    let d = r.unwrap();
    assert_eq!(d.texto, "Despacho do órgão de origem.");
    assert_eq!(d.anexo.unwrap().mime, "text/html");
}

#[test]
fn com_saida_o_parcial_fica_no_diretorio_ate_concluir() {
    let amb = ambiente();
    let dir = std::env::temp_dir().join(format!("ies-externo-saida-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let pedido = PedidoDoAnexo {
        cache_host: None,
        limite: LimiteDoAnexo::default(),
        saida: Some(&dir),
    };
    let (r, _) = rodar(&amb, pdf(), pedido);
    let mut anexo = r.unwrap().anexo.unwrap();
    let arquivo = anexo.tirar_arquivo().expect("parcial");
    assert_eq!(arquivo.nome(), "0000101.pdf");
    arquivo.concluir().unwrap();
    assert_eq!(
        std::fs::read(dir.join("0000101.pdf")).unwrap(),
        b"%PDF-1.7 abc"
    );
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn limite_do_pedido_chega_ao_transporte_e_recusa() {
    let amb = ambiente();
    let pedido = PedidoDoAnexo {
        cache_host: None,
        limite: LimiteDoAnexo::novo(10).unwrap(),
        saida: None,
    };
    let (r, envio) = rodar(&amb, pdf(), pedido);
    assert_eq!(envio.limite_pedido, Some(10));
    assert_eq!(
        r.unwrap_err(),
        ErroDocumentoObter::Anexo(ErroAnexo::AcimaDoLimite)
    );
}

#[test]
fn redirect_para_o_login_e_sessao_encerrada_e_tipo_fora_da_lista_e_recusado() {
    let amb = ambiente();
    let (r, _) = rodar(
        &amb,
        binaria(302, &[("location", LOGIN)], b""),
        PedidoDoAnexo::default(),
    );
    assert_eq!(
        r.unwrap_err(),
        ErroDocumentoObter::Processo(ErroProcessoObter::SessaoEncerrada(
            crate::erro_publico::SessaoEncerrada
        ))
    );
    let amb = ambiente();
    let (r, _) = rodar(
        &amb,
        binaria(200, &[("content-type", "application/zip")], b"PK"),
        PedidoDoAnexo::default(),
    );
    assert_eq!(
        r.unwrap_err(),
        ErroDocumentoObter::Anexo(ErroAnexo::TipoNaoPermitido)
    );
    let amb = ambiente();
    let login = "<html><body><form id=\"frmLogin\"><input id=\"txtUsuario\" name=\"txtUsuario\">\
        <input type=\"password\" id=\"pwdSenha\" name=\"pwdSenha\"></form></body></html>";
    let (r, _) = rodar(
        &amb,
        binaria(200, &[("content-type", "text/html")], login.as_bytes()),
        PedidoDoAnexo::default(),
    );
    assert_eq!(
        r.unwrap_err(),
        ErroDocumentoObter::Processo(ErroProcessoObter::SessaoEncerrada(
            crate::erro_publico::SessaoEncerrada
        ))
    );
}

/// Fatia 2 + 3: o índice já sabe o número e guarda o link da árvore; o
/// externo custa o mesmo que o interno por esse caminho, com o GET
/// binário no lugar do GET do conteúdo.
#[test]
fn externo_pelo_link_da_arvore_guardado_em_duas() {
    let amb = ambiente();
    let (inst, g) = (instalacao(), geracao());
    amb.indice.registrar_processo(&inst, NUP, ID).unwrap();
    amb.indice
        .registrar_documentos(&inst, ID, &[(DOC, ID_DOC)])
        .unwrap();
    amb.indice
        .registrar_link(
            &inst,
            chave(&g, AcaoDoSei::ProcedimentoVisualizar, ID),
            &url_arvore(ID),
        )
        .unwrap();
    let arvore = ate_a_arvore().pop().unwrap();
    let (r, envio) = rodar_com(&amb, vec![arvore], pdf(), PedidoDoAnexo::default());
    let d = r.unwrap();
    assert_eq!(
        envio.roteiro.pedidos,
        [get(&url_arvore(ID)), get(&absoluta(&src_anexo()))]
    );
    assert_eq!(d.requisicoes, 2);
    assert_eq!(d.resolvido_por, ResolvidoPor::IndiceLink);
    assert_eq!((d.id_documento.as_str(), d.unidade), (ID_DOC, None));
    assert_eq!(d.anexo.expect("anexo").mime, "application/pdf");
}
