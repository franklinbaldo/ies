//! A sequência que `ies processo buscar` percorre, sobre as respostas
//! gravadas de uma cassette: Controle (unidade e link do menu), GET da tela
//! (POST derivado), envio (a resposta gravada entra no lugar do POST) e o
//! plano da página 2. O que a gravação não estabelece (unidade antes da
//! tela, tela antes do envio, corpo do POST) é `nao_aplicavel`.
use super::{Leitor, Registro, Resultado};
use crate::cassette::acervo::gravacao::{self, Resposta};
use crate::controle_reconhecido::unidade_do_controle;
use crate::erro_publico::Rotulo as _;
use crate::formulario_web::{
    RespostaDoEnvio, RespostaDoPost, SubmissaoDerivada, TelaServida, enviar_submissao_via,
};
use crate::html_atributos::{atributo_presente, nos_com_id};
use crate::instalacao::Instalacao;
use crate::pagina_do_controle::codificacao_da_tela;
use crate::pesquisa_web::erro_busca::ErroBuscaDeProcesso;
use crate::pesquisa_web::formulario_pesquisa::{
    derivar_pagina_da_pesquisa, derivar_pesquisa_de_protocolo, falha_do_envio, interpretar_envio,
};
use crate::pesquisa_web::link_da_pesquisa::link_da_pesquisa;
use crate::pesquisa_web::paginacao::{ErroPaginacaoDaPesquisa, planejar_pagina};
use crate::pesquisa_web::resposta::{RespostaDaPesquisa, ler_resposta_html};
use crate::transporte::sessao::CookieDaSessao;
use crate::transporte::{RespostaWeb, decodificar_resposta};
use std::path::PathBuf;

/// O termo que a derivação recebe: o corpo do POST gravado não existe, e o
/// termo não muda a resposta já gravada; só precisa caber no `maxlength`.
pub(super) const TERMO_DE_PROVA: &str = "0";

mod documentos;

/// O que a gravação estabeleceu até o passo atual.
#[derive(Default)]
struct Estado {
    em_documentos: documentos::EstadoEmDocumentos,
    unidade: Option<String>,
    pendente: Option<SubmissaoDerivada>,
    /// A última resposta da Pesquisa foi uma tela de resultado (200 do POST).
    na_tela_de_resultado: bool,
}

struct Cassette<'a> {
    card: &'a str,
    instalacao_nome: &'a str,
    instalacao: Instalacao,
    cookie: &'a CookieDaSessao,
}

impl Cassette<'_> {
    fn registro(&self, leitor: Leitor, passo: usize, resultado: Resultado) -> Registro {
        Registro {
            leitor,
            instalacao: self.instalacao_nome.to_owned(),
            card: self.card.to_owned(),
            passo,
            resultado,
            forma: None,
            total_declarado: None,
            faixa: None,
            linhas_ausentes: 0,
        }
    }

    fn lido(&self, leitor: Leitor, passo: usize, forma: &'static str) -> Registro {
        let mut r = self.registro(leitor, passo, Resultado::Lido);
        r.forma = Some(forma);
        r
    }
}

pub(super) fn replay(card: &str, instalacao_nome: &str, json: &serde_json::Value) -> Vec<Registro> {
    let Some(instalacao) = gravacao::instalacao(json) else {
        return Vec::new();
    };
    let cookie = CookieSintetico::novo();
    let lido = CookieDaSessao::ler(&cookie.0);
    let c = Cassette {
        card,
        instalacao_nome,
        instalacao,
        cookie: &lido,
    };
    let mut estado = Estado::default();
    let mut registros = Vec::new();
    for r in gravacao::respostas(json) {
        match (r.operacao, r.metodo) {
            ("procedimento_controlar", "GET") => registros.push(controle(&c, &r, &mut estado)),
            ("protocolo_pesquisar", "GET") => {
                registros.push(tela(&c, &r, &mut estado));
                let unidade = estado.unidade.as_deref();
                let d = &mut estado.em_documentos;
                registros.push(documentos::tela(&c, &r, unidade, d));
            }
            ("protocolo_pesquisar", "POST") => {
                let unidade = estado.unidade.clone();
                let d = &mut estado.em_documentos;
                registros.extend(documentos::envio(&c, &r, unidade.as_deref(), d));
                registros.extend(envio(&c, &r, &mut estado));
            }
            _ => {}
        }
    }
    registros
}

fn url(r: &Resposta<'_>) -> String {
    r.item["response"]["url"].as_str().unwrap_or("").to_owned()
}

fn web(r: &Resposta<'_>) -> RespostaWeb<std::io::Cursor<Vec<u8>>> {
    RespostaWeb {
        status_code: i32::try_from(r.status).unwrap_or(0),
        mime: gravacao::mime(r.item),
        content_length: None,
        localizacao: None,
        corpo: std::io::Cursor::new(gravacao::corpo(r.item)),
    }
}

/// Requisição 1 do comando: transporte, unidade reconhecida e link do menu.
fn controle(c: &Cassette<'_>, r: &Resposta<'_>, estado: &mut Estado) -> Registro {
    let lido = decodificar_resposta(web(r))
        .map_err(|f| f.rotulo())
        .and_then(|html| {
            let unidade = unidade_do_controle(&html).map_err(|e| e.rotulo())?;
            link_da_pesquisa(&c.instalacao, &url(r), &html).map_err(|e| e.rotulo())?;
            Ok(unidade)
        });
    match lido {
        Ok(unidade) => {
            estado.unidade = Some(unidade);
            c.registro(Leitor::Controle, r.passo, Resultado::Lido)
        }
        Err(sub) => c.registro(Leitor::Controle, r.passo, Resultado::Recusado(sub)),
    }
}

/// Requisição 2: a tela servida e o POST derivado dela (guardado para o envio).
fn tela(c: &Cassette<'_>, r: &Resposta<'_>, estado: &mut Estado) -> Registro {
    estado.pendente = None;
    estado.na_tela_de_resultado = false;
    let Some(unidade) = estado.unidade.as_deref() else {
        let motivo = Resultado::NaoAplicavel("unidade_do_controle_nao_gravada_antes_da_tela");
        return c.registro(Leitor::Tela, r.passo, motivo);
    };
    let derivada = derivar(r, &url(r), |tela| {
        derivar_pesquisa_de_protocolo(&c.instalacao, tela, unidade, TERMO_DE_PROVA)
            .map_err(|e| e.rotulo())
    });
    match derivada {
        Ok(submissao) => {
            estado.pendente = Some(submissao);
            c.registro(Leitor::Tela, r.passo, Resultado::Lido)
        }
        Err(sub) => c.registro(Leitor::Tela, r.passo, Resultado::Recusado(sub)),
    }
}

/// A tela gravada em `r`, servida em `url` (a do GET, ou a do POST para a
/// tela de resultado, como no fluxo), entregue à derivação.
fn derivar<T>(
    r: &Resposta<'_>,
    url: &str,
    f: impl FnOnce(&TelaServida<'_>) -> Result<T, &'static str>,
) -> Result<T, &'static str> {
    let mime = gravacao::mime(r.item);
    let html = decodificar_resposta(web(r)).map_err(|f| f.rotulo())?;
    let codificacao = codificacao_da_tela(&mime)
        .ok_or(ErroBuscaDeProcesso::CharsetDaTelaNaoSuportado.rotulo())?;
    f(&TelaServida {
        url,
        html: &html,
        codificacao,
    })
}

/// Requisição 3: a resposta gravada no lugar do POST derivado.
fn envio(c: &Cassette<'_>, r: &Resposta<'_>, estado: &mut Estado) -> Vec<Registro> {
    let Some(submissao) = estado.pendente.take() else {
        let (leitor, motivo) = if estado.na_tela_de_resultado {
            (
                Leitor::PostDaTelaDeResultado,
                "post_da_tela_de_resultado_sem_corpo_gravado",
            )
        } else {
            (
                Leitor::Envio,
                "tela_da_pesquisa_nao_derivada_antes_do_envio",
            )
        };
        return vec![c.registro(leitor, r.passo, Resultado::NaoAplicavel(motivo))];
    };
    let url_do_envio = submissao.url().to_owned();
    let gravada = RespostaDoPost {
        resposta: web(r),
        localizacao: gravacao::localizacao(r.item).map(str::to_owned),
    };
    let enviada = match enviar_submissao_via(submissao, c.cookie, |_, _| Ok(gravada)) {
        Ok(enviada) => enviada,
        Err(erro) => {
            let sub = crate::erro_publico::rotulo(&falha_do_envio(erro));
            return vec![c.registro(Leitor::Envio, r.passo, Resultado::Recusado(sub))];
        }
    };
    let opcao = match &enviada {
        RespostaDoEnvio::Html(html) => {
            estado.na_tela_de_resultado = true;
            opcao_marcada(html)
        }
        RespostaDoEnvio::Redirecionamento(_) => Opcao::Processos,
    };
    if opcao == Opcao::Documentos {
        return vec![em_documentos(c, r, &enviada)];
    }
    let interpretado = match interpretar_envio(enviada, &c.instalacao) {
        Ok(i) => i,
        Err(erro) => {
            let sub = erro.rotulo();
            return vec![c.registro(Leitor::Envio, r.passo, Resultado::Recusado(sub))];
        }
    };
    if opcao == Opcao::Indeterminada {
        let motivo = Resultado::NaoAplicavel("opcao_pesquisar_em_nao_observada_na_resposta");
        return vec![c.registro(Leitor::Envio, r.passo, motivo)];
    }
    match interpretado.resposta {
        RespostaDaPesquisa::ProcessoUnico { .. } => {
            vec![c.lido(Leitor::Envio, r.passo, "processo_unico")]
        }
        RespostaDaPesquisa::SemResultado => vec![c.lido(Leitor::Envio, r.passo, "sem_resultado")],
        RespostaDaPesquisa::Lista(ref lista) => {
            let forma = if lista.linhas_ausentes_da_faixa > 0 {
                "lista_com_linhas_ausentes"
            } else {
                "lista"
            };
            let mut lido = c.lido(Leitor::Envio, r.passo, forma);
            lido.total_declarado = Some(lista.total_declarado);
            lido.faixa = Some((lista.faixa.inicio, lista.faixa.fim));
            lido.linhas_ausentes = lista.linhas_ausentes_da_faixa;
            let unidade = estado.unidade.as_deref().unwrap_or_default();
            let plano = plano(c, r, &interpretado.resposta, &url_do_envio, unidade);
            vec![lido, plano]
        }
    }
}

/// Pesquisa em documentos: as linhas não são de processo (o comando só
/// envia `P`); o aviso de "sem resultado" é o mesmo e é lido.
fn em_documentos(c: &Cassette<'_>, r: &Resposta<'_>, enviada: &RespostaDoEnvio) -> Registro {
    let RespostaDoEnvio::Html(html) = enviada else {
        unreachable!("só o 200 publica a opção marcada")
    };
    match ler_resposta_html(html) {
        Ok(RespostaDaPesquisa::SemResultado) => {
            c.lido(Leitor::Envio, r.passo, "sem_resultado_em_documentos")
        }
        _ => c.registro(
            Leitor::Envio,
            r.passo,
            Resultado::NaoAplicavel("lista_da_pesquisa_em_documentos"),
        ),
    }
}

/// `--pagina 2` sobre a página 1 gravada: regra `navegar()`, links e o
/// POST de página derivado da tela de resultado (sem enviá-lo).
fn plano(
    c: &Cassette<'_>,
    r: &Resposta<'_>,
    resposta: &RespostaDaPesquisa,
    url_do_envio: &str,
    unidade: &str,
) -> Registro {
    let html = decodificar_resposta(web(r)).ok();
    let pedido = match planejar_pagina(resposta, html.as_deref(), 2) {
        Ok(Some(pedido)) => pedido,
        Ok(None) => unreachable!("página 2 nunca é a página 1"),
        Err(ErroPaginacaoDaPesquisa::PaginaForaDoIntervalo { paginas: 1, .. }) => {
            return c.lido(Leitor::PlanoDaPagina, r.passo, "uma_pagina_so");
        }
        Err(erro) => {
            let sub = erro.rotulo();
            return c.registro(Leitor::PlanoDaPagina, r.passo, Resultado::Recusado(sub));
        }
    };
    let derivada = derivar(r, url_do_envio, |tela| {
        derivar_pagina_da_pesquisa(&c.instalacao, tela, unidade, TERMO_DE_PROVA, pedido.inicio)
            .map_err(|e| e.rotulo())
    });
    match derivada {
        Ok(_) => c.lido(Leitor::PlanoDaPagina, r.passo, "pagina_2_planejada"),
        Err(sub) => c.registro(Leitor::PlanoDaPagina, r.passo, Resultado::Recusado(sub)),
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Opcao {
    Processos,
    Documentos,
    Indeterminada,
}

/// O rádio `rdoPesquisarEm` marcado na tela de resultado (por
/// `html_atributos`): exatamente um de `#optProcessos` e `#optDocumentos`.
fn opcao_marcada(html: &str) -> Opcao {
    let Ok(dom) = tl::parse(html, tl::ParserOptions::default()) else {
        return Opcao::Indeterminada;
    };
    let marcado = |id: &str| -> Option<bool> {
        let nos = nos_com_id(&dom, id).ok()?;
        let [no] = nos.as_slice() else {
            return None;
        };
        let tag = no.get(dom.parser())?.as_tag()?;
        atributo_presente(tag, "checked").ok()
    };
    match (marcado("optProcessos"), marcado("optDocumentos")) {
        (Some(true), Some(false)) => Opcao::Processos,
        (Some(false), Some(true)) => Opcao::Documentos,
        _ => Opcao::Indeterminada,
    }
}

/// Arquivo de cookie `0600` sintético: o envio o lê, a resposta gravada
/// não depende dele. Removido ao fim da cassette.
struct CookieSintetico(PathBuf);

impl CookieSintetico {
    fn novo() -> Self {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static N: AtomicUsize = AtomicUsize::new(0);
        let pasta = std::env::temp_dir().join(format!(
            "ies-acervo-pesquisa-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&pasta).expect("pasta temporária");
        let arquivo = pasta.join("cookie");
        std::fs::write(&arquivo, "PHPSESSID=SINTETICO\n").expect("cookie sintético");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let modo = std::fs::Permissions::from_mode(0o600);
            std::fs::set_permissions(&arquivo, modo).expect("0600");
        }
        Self(arquivo)
    }
}

impl Drop for CookieSintetico {
    fn drop(&mut self) {
        if let Some(pasta) = self.0.parent() {
            let _ = std::fs::remove_dir_all(pasta);
        }
    }
}
