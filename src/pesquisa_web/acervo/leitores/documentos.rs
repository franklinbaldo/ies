//! As mesmas respostas gravadas lidas pelas peças de `ies documento buscar`
//! (Refs #61): o POST em documentos derivado da tela gravada, o envio (302
//! com `id_documento` ou 200 com linhas de documento), o plano da página 2
//! e, nos POSTs da tela de resultado (`navegar()`, sem corpo gravado), a
//! página seguinte lida e conferida contra o plano feito da página 1.
use super::{Cassette, Opcao, derivar, opcao_marcada, url, web};
use crate::cassette::acervo::gravacao::{self, Resposta};
use crate::erro_publico::Rotulo as _;
use crate::formulario_web::{
    RespostaDoEnvio, RespostaDoPost, SubmissaoDerivada, enviar_submissao_via,
};
use crate::pesquisa_web::acervo::{Leitor, Registro, Resultado};
use crate::pesquisa_web::alvo::AlvoDaPesquisa;
use crate::pesquisa_web::documentos::{
    EmDocumentos, RespostaDeDocumentos, ler_resposta_de_documentos,
};
use crate::pesquisa_web::formulario_pesquisa::{
    ErroPesquisaProtocolo, derivar_pagina_em, derivar_pesquisa_em, falha_do_envio,
};
use crate::pesquisa_web::paginacao::{
    Contagem, ErroPaginacaoDaPesquisa, conferir_contagem, planejar_pagina_por,
};
use crate::pesquisa_web::redirecionamento::ErroRedirecionamentoDaPesquisa as R;
use crate::pesquisa_web::regras_formulario::PesquisarEm;
use crate::pesquisa_web::resposta::pagina_selecionada;
use crate::transporte::decodificar_resposta;

/// O termo derivado: só precisa caber no `maxlength` (a resposta é a gravada).
const TERMO: &str = "0";

#[derive(Default)]
pub(super) struct EstadoEmDocumentos {
    pendente: Option<SubmissaoDerivada>,
    /// Página 1 lida (contagem e HTML) da última pesquisa em documentos.
    pagina_1: Option<(Contagem, String)>,
    na_tela_de_resultado: bool,
}

/// GET da tela: o POST em documentos derivado (guardado para o envio).
pub(super) fn tela(
    c: &Cassette<'_>,
    r: &Resposta<'_>,
    unidade: Option<&str>,
    d: &mut EstadoEmDocumentos,
) -> Registro {
    *d = EstadoEmDocumentos::default();
    let Some(unidade) = unidade else {
        let motivo = Resultado::NaoAplicavel("unidade_do_controle_nao_gravada_antes_da_tela");
        return c.registro(Leitor::TelaEmDocumentos, r.passo, motivo);
    };
    let derivada = derivar(r, &url(r), |tela| {
        derivar_pesquisa_em(
            PesquisarEm::Documentos,
            &c.instalacao,
            tela,
            (unidade, TERMO),
        )
        .map_err(|e| e.rotulo())
    });
    match derivada {
        Ok(submissao) => {
            d.pendente = Some(submissao);
            c.registro(Leitor::TelaEmDocumentos, r.passo, Resultado::Lido)
        }
        Err(sub) => c.registro(Leitor::TelaEmDocumentos, r.passo, Resultado::Recusado(sub)),
    }
}

/// POST: logo depois da tela, o envio; na tela de resultado, a página seguinte.
pub(super) fn envio(
    c: &Cassette<'_>,
    r: &Resposta<'_>,
    unidade: Option<&str>,
    d: &mut EstadoEmDocumentos,
) -> Vec<Registro> {
    match d.pendente.take() {
        Some(submissao) => primeiro_envio(c, r, unidade.unwrap_or_default(), submissao, d),
        None if d.na_tela_de_resultado => pagina_seguinte(c, r, d).into_iter().collect(),
        None => Vec::new(),
    }
}

fn primeiro_envio(
    c: &Cassette<'_>,
    r: &Resposta<'_>,
    unidade: &str,
    submissao: SubmissaoDerivada,
    d: &mut EstadoEmDocumentos,
) -> Vec<Registro> {
    let url_do_envio = submissao.url().to_owned();
    let gravada = RespostaDoPost {
        resposta: web(r),
        localizacao: gravacao::localizacao(r.item).map(str::to_owned),
    };
    let leitor = Leitor::EnvioEmDocumentos;
    let enviada = match enviar_submissao_via(submissao, c.cookie, |_, _| Ok(gravada)) {
        Ok(enviada) => enviada,
        Err(erro) => {
            let sub = crate::erro_publico::rotulo(&falha_do_envio(erro));
            return vec![c.registro(leitor, r.passo, Resultado::Recusado(sub))];
        }
    };
    if let RespostaDoEnvio::Html(html) = &enviada {
        d.na_tela_de_resultado = true;
        if opcao_marcada(html) != Opcao::Documentos {
            d.na_tela_de_resultado = false;
            let motivo = Resultado::NaoAplicavel("pesquisa_gravada_em_processos");
            return vec![c.registro(leitor, r.passo, motivo)];
        }
    }
    let interpretado = match EmDocumentos::interpretar(enviada, &c.instalacao) {
        Ok(i) => i,
        Err(ErroPesquisaProtocolo::Redirecionamento(R::IdDocumentoAusenteNoRedirecionamento)) => {
            let motivo = Resultado::NaoAplicavel("redirecionamento_para_processo_sem_id_documento");
            return vec![c.registro(leitor, r.passo, motivo)];
        }
        Err(erro) => {
            let sub = erro.rotulo();
            return vec![c.registro(leitor, r.passo, Resultado::Recusado(sub))];
        }
    };
    match interpretado.resposta {
        RespostaDeDocumentos::DocumentoUnico { .. } => {
            vec![c.lido(leitor, r.passo, "documento_unico")]
        }
        RespostaDeDocumentos::SemResultado => vec![c.lido(leitor, r.passo, "sem_resultado")],
        RespostaDeDocumentos::Lista(ref lista) => {
            let forma = if lista.linhas_ausentes_da_faixa > 0 {
                "lista_com_linhas_ausentes"
            } else {
                "lista"
            };
            let mut lido = c.lido(leitor, r.passo, forma);
            lido.total_declarado = Some(lista.total_declarado);
            let contagem = EmDocumentos::contagem(&interpretado.resposta);
            let html = interpretado.html.unwrap_or_default();
            let plano = plano(c, r, contagem, &html, &url_do_envio, unidade);
            d.pagina_1 = Some((contagem, html));
            vec![lido, plano]
        }
    }
}

/// `--pagina 2` sobre a página 1: regra `navegar()` e o POST derivado.
fn plano(
    c: &Cassette<'_>,
    r: &Resposta<'_>,
    contagem: Contagem,
    html: &str,
    url_do_envio: &str,
    unidade: &str,
) -> Registro {
    let leitor = Leitor::PlanoEmDocumentos;
    let pedido = match planejar_pagina_por(contagem, Some(html), 2) {
        Ok(Some(pedido)) => pedido,
        Ok(None) => unreachable!("página 2 nunca é a página 1"),
        Err(ErroPaginacaoDaPesquisa::PaginaForaDoIntervalo { paginas: 1, .. }) => {
            return c.lido(leitor, r.passo, "uma_pagina_so");
        }
        Err(erro) => {
            return c.registro(leitor, r.passo, Resultado::Recusado(erro.rotulo()));
        }
    };
    let derivada = derivar(r, url_do_envio, |tela| {
        let pedido_da_tela = (unidade, TERMO);
        derivar_pagina_em(
            PesquisarEm::Documentos,
            &c.instalacao,
            tela,
            pedido_da_tela,
            pedido.inicio,
        )
        .map_err(|e| e.rotulo())
    });
    match derivada {
        Ok(_) => c.lido(leitor, r.passo, "pagina_2_planejada"),
        Err(sub) => c.registro(leitor, r.passo, Resultado::Recusado(sub)),
    }
}

/// POST de `navegar()` gravado sem corpo: a página pedida é a que o
/// paginador marca; o plano é feito da página 1 e a resposta, lida com a
/// faixa esperada e conferida, como no fluxo do comando.
fn pagina_seguinte(
    c: &Cassette<'_>,
    r: &Resposta<'_>,
    d: &mut EstadoEmDocumentos,
) -> Option<Registro> {
    let leitor = Leitor::PaginaEmDocumentos;
    let html = decodificar_resposta(web(r)).ok()?;
    if opcao_marcada(&html) != Opcao::Documentos {
        return None;
    }
    let recusa = |sub| Some(c.registro(leitor, r.passo, Resultado::Recusado(sub)));
    let Some((primeira, html_1)) = &d.pagina_1 else {
        return match ler_resposta_de_documentos(&html, None) {
            Ok(_) => Some(c.lido(leitor, r.passo, "pagina_lida_sem_pagina_1")),
            Err(erro) => recusa(erro.rotulo()),
        };
    };
    let dom = tl::parse(&html, tl::ParserOptions::default()).ok()?;
    let numero = match pagina_selecionada(&dom) {
        Ok(Some(n)) => u32::try_from(n).unwrap_or(u32::MAX),
        _ => return recusa("pagina_selecionada_nao_observada_no_paginador"),
    };
    let pedido = match planejar_pagina_por(*primeira, Some(html_1), numero) {
        Ok(Some(pedido)) => pedido,
        Ok(None) => return Some(c.lido(leitor, r.passo, "pagina_1_reenviada")),
        Err(erro) => return recusa(erro.rotulo()),
    };
    let resposta = match ler_resposta_de_documentos(&html, Some((pedido.esperada, pedido.numero))) {
        Ok(resposta) => resposta,
        Err(erro) => return recusa(erro.rotulo()),
    };
    let forma = match &resposta {
        RespostaDeDocumentos::Lista(l) if l.linhas_ausentes_da_faixa > 0 => {
            "pagina_seguinte_conferida_com_linhas_ausentes"
        }
        _ => "pagina_seguinte_conferida",
    };
    Some(
        match conferir_contagem(&pedido, EmDocumentos::contagem(&resposta)) {
            Ok(()) => c.lido(leitor, r.passo, forma),
            Err(erro) => c.registro(leitor, r.passo, Resultado::Recusado(erro.rotulo())),
        },
    )
}
