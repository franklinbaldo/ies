//! Relações entre processos: prévia, escrita única e conferência independente.
mod escrita;
mod tela;
#[cfg(test)]
#[path = "processo_relacionar/tests.rs"]
mod tests;
#[cfg(test)]
#[path = "processo_relacionar/tests_escrita.rs"]
mod tests_escrita;
#[cfg(test)]
#[path = "processo_relacionar/tests_fluxo.rs"]
mod tests_fluxo;

use crate::arvore_processo::ErroRelacionarNaBarra;
use crate::formulario_web::{
    ConsultaAjax, EfeitoAvaliado, ErroSubmissaoFormulario, TelaServida, derivar_consulta_ajax,
};
use crate::indice_local::IndiceLocal;
use crate::instalacao::Instalacao;
use crate::pagina_do_controle::codificacao_da_tela;
use crate::processo::NumeroProcesso;
use crate::processo_concluir::escrita::CookieIlegivel;
use crate::processo_concluir::escrita::{Contador, FalhaDaTela, leitura, ler_cookie, obter_tela};
use crate::processo_obter::{
    EnvioDoProcesso, ErroProcessoObter, SessaoDoProcesso, abrir_para_escrever,
};
use crate::transporte::{FalhaTransporte, RespostaWeb, ler_limitado, recusar_status};
use crate::visao_do_controle::EnvioDoControle as _;
use std::io::Read;
use tela::{
    DestinoConsultado, ErroDestinoDaConsulta, ErroTelaDeRelacionamentos, Relacionamento,
    ler_destino, ler_linhas, url_complementar,
};

const CONSULTA: ConsultaAjax = ConsultaAjax {
    acao_ajax: "protocolo_RI1023",
    campos: &["idProcedimento"],
    efeito: EfeitoAvaliado::ConsultaSemEfeitoObservado {
        evidencia: "tests/paginas_reais/procedimento_relacionar.html no Todos @323eb01: consulta do botão Pesquisar",
    },
};

#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum FalhaRelacionar {
    /// O cookie privado da sessão não pôde ser lido.
    Cookie(CookieIlegivel),
    /// Caminho de escrita implementado offline, mas ainda sem validação live.
    Abertura(ErroProcessoObter),
    Barra(ErroRelacionarNaBarra),
    LinkNaoPublicado,
    Tela(FalhaTransporte),
    TelaRedirecionada,
    SessaoEncerrada,
    TelaDeRelacionamentos(#[from] ErroTelaDeRelacionamentos),
    RelacionamentosRepetidosNaTela,
    DestinoARemoverSemIdentificador,
    CharsetDaTelaDeRelacionamentosNaoSuportado,
    ConsultaComplementarDeRelacionamentoNaoDeclarada,
    ConsultaDeRelacionamentoRedirecionada,
    DestinoDaConsulta(#[from] ErroDestinoDaConsulta),
    DestinoARelacionarNaoIdentificado,
    AcaoExcluirRelacionamentoNaoDeclarada,
    IdDaOrigemNaActionAusente,
    ActionDeRemocaoDeOutroProcesso,
    ConsultaDerivacao(ErroSubmissaoFormulario),
    ConsultaTransporte(FalhaTransporte),
    ConsultaResposta(FalhaTransporte),
    RelacionamentoConsigoMesmo,
    RemocaoNaoOferecida,
    DestinoDaConsultaDivergente,
    ConsultaNaoXml,
    /// O POST pode ter sido enviado; causa preservada quando disponível.
    NaoConfirmado(Option<ErroSubmissaoFormulario>),
}
pub(crate) struct Pedido<'a> {
    pub(crate) nup: &'a NumeroProcesso,
    pub(crate) com: &'a NumeroProcesso,
    pub(crate) unidade: Option<&'a str>,
    pub(crate) remover: bool,
    pub(crate) confirmar: bool,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ResultadoRelacionar {
    Previa,
    SemEfeito,
    Confirmado,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Previa {
    pub(crate) nup: String,
    pub(crate) id_procedimento: String,
    pub(crate) destino: String,
    pub(crate) id_destino: Option<String>,
    pub(crate) tipo_destino: Option<String>,
    pub(crate) remover: bool,
    pub(crate) resultado: ResultadoRelacionar,
    pub(crate) requisicoes: u8,
}

impl Previa {
    pub(crate) fn sem_efeito(&self) -> bool {
        self.resultado == ResultadoRelacionar::SemEfeito
    }
}

/// Números de processo são opacos; para buscar vínculos, diferenças de
/// pontuação e caixa não mudam a sequência de letras e dígitos. Oráculo:
/// Todos `tipos.mesmo_protocolo` e `test_so_os_digitos_bastam` @323eb01.
fn forma_do_protocolo(numero: &str) -> String {
    numero
        .chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

pub(crate) fn mesmo_protocolo(pedido: &str, mostrado: &str) -> bool {
    let forma = forma_do_protocolo(pedido);
    !forma.is_empty() && forma == forma_do_protocolo(mostrado)
}

/// O texto exato tem prioridade; dois equivalentes sem texto exato
/// são ambíguos (AB-C e A-BC não permitem escolher por acaso).
fn linhas_do_protocolo<'a>(
    relacionados: &'a [Relacionamento],
    nup: &str,
) -> Vec<&'a Relacionamento> {
    let literal = nup.trim().to_lowercase();
    let exatas: Vec<_> = relacionados
        .iter()
        .filter(|linha| linha.nup.trim().to_lowercase() == literal)
        .collect();
    if !exatas.is_empty() {
        return exatas;
    }
    relacionados
        .iter()
        .filter(|linha| mesmo_protocolo(nup, &linha.nup))
        .collect()
}

/// Só uma linha do formulário pode comprovar o vínculo, sem consulta AJAX.
fn relacionamento_existente<'a>(
    relacionados: &'a [Relacionamento],
    nup: &str,
    id_origem: &str,
) -> Result<Option<&'a Relacionamento>, FalhaRelacionar> {
    match linhas_do_protocolo(relacionados, nup).as_slice() {
        [] => Ok(None),
        [linha] if linha.id == id_origem => Err(FalhaRelacionar::RelacionamentoConsigoMesmo),
        [linha] => Ok(Some(*linha)),
        _ => Err(FalhaRelacionar::RelacionamentosRepetidosNaTela),
    }
}

/// A consulta do protocolo é XML (e não a tela HTML do controle).
/// Não reutilizar o decodificador exclusivo de text/html da infraestrutura.
fn xml_da_consulta<R: Read>(mut resposta: RespostaWeb<R>) -> Result<String, FalhaRelacionar> {
    if resposta.status_code != 200 {
        return Err(FalhaRelacionar::ConsultaResposta(recusar_status(resposta)));
    }
    let tipo = resposta.mime.split(';').next().unwrap_or_default().trim();
    if !tipo.eq_ignore_ascii_case("application/xml") && !tipo.eq_ignore_ascii_case("text/xml") {
        return Err(FalhaRelacionar::ConsultaNaoXml);
    }
    if resposta
        .content_length
        .is_some_and(|tamanho| tamanho > crate::MAX_HTML_BYTES)
    {
        return Err(FalhaRelacionar::ConsultaResposta(
            FalhaTransporte::RespostaExcessiva,
        ));
    }
    let bytes = ler_limitado(&mut resposta.corpo, crate::MAX_HTML_BYTES)
        .map_err(FalhaRelacionar::ConsultaResposta)?;
    crate::transporte::corpo::decode_http_body(&bytes, &resposta.mime).map_err(|_| {
        FalhaRelacionar::ConsultaResposta(FalhaTransporte::CorpoNaoDecodificavelNoCharsetDeclarado)
    })
}

/// A confirmação percorre a prévia antes da escrita única; em ambas as
/// ações o POST deve publicar aceite e uma nova leitura comprovar o efeito.
#[allow(clippy::too_many_lines)] // Fluxo de prévia por ação com contagem, sem mutação nem retry.
pub(crate) fn prever<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    pedido: &Pedido<'_>,
    indice: &IndiceLocal,
    envio: &mut V,
) -> Result<Previa, FalhaRelacionar> {
    let instalacao = Instalacao::interpretar(sessao.origem)
        .ok_or(FalhaRelacionar::Tela(FalhaTransporte::OrigemNaoAutorizada))?;
    let mut rede = Contador::novo(envio);
    let (aberto, acoes) = abrir_para_escrever(
        sessao,
        &leitura(pedido.nup, pedido.unidade, true),
        indice,
        &mut rede,
    )
    .map_err(FalhaRelacionar::Abertura)?;
    let id = aberto.arvore.processo.id;
    let link = acoes
        .relacionar
        .map_err(FalhaRelacionar::Barra)?
        .ok_or(FalhaRelacionar::LinkNaoPublicado)?;
    let cookie = ler_cookie(sessao)?;
    let (html, mime) = obter_tela(&mut rede, &cookie, link.as_str()).map_err(|e| match e {
        FalhaDaTela::Transporte(falha) => FalhaRelacionar::Tela(falha),
        FalhaDaTela::Redirecionada => FalhaRelacionar::TelaRedirecionada,
        FalhaDaTela::SessaoEncerrada(crate::erro_publico::SessaoEncerrada) => {
            FalhaRelacionar::SessaoEncerrada
        }
    })?;
    let relacionados = ler_linhas(&html)?;
    if mesmo_protocolo(pedido.nup.as_str(), pedido.com.as_str()) {
        return Err(FalhaRelacionar::RelacionamentoConsigoMesmo);
    }
    let mut previa = Previa {
        nup: pedido.nup.as_str().to_owned(),
        id_procedimento: id.clone(),
        destino: pedido.com.as_str().to_owned(),
        id_destino: None,
        tipo_destino: None,
        remover: pedido.remover,
        resultado: ResultadoRelacionar::Previa,
        requisicoes: rede.feitas(),
    };
    if pedido.remover {
        let achados = linhas_do_protocolo(&relacionados, pedido.com.as_str());
        match achados.as_slice() {
            [] => previa.resultado = ResultadoRelacionar::SemEfeito,
            [linha] => {
                if !linha.pode_remover {
                    return Err(FalhaRelacionar::RemocaoNaoOferecida);
                }
                previa.destino.clone_from(&linha.nup);
                previa.id_destino = Some(linha.id.clone());
            }
            _ => {
                return Err(FalhaRelacionar::RelacionamentosRepetidosNaTela);
            }
        }
        if pedido.confirmar && !previa.sem_efeito() {
            let id_destino = previa
                .id_destino
                .as_deref()
                .ok_or(FalhaRelacionar::DestinoARemoverSemIdentificador)?;
            escrita::remover(
                sessao,
                &instalacao,
                &cookie,
                link.as_str(),
                &html,
                &mime,
                id_destino,
                &relacionados,
                &mut rede,
            )?;
            previa.resultado = ResultadoRelacionar::Confirmado;
        }
        previa.requisicoes = rede.feitas();
        return Ok(previa);
    }
    // A tabela já relaciona NUP e id: nenhum POST AJAX é necessário.
    if let Some(linha) = relacionamento_existente(&relacionados, pedido.com.as_str(), &id)? {
        previa.id_destino = Some(linha.id.clone());
        previa.destino.clone_from(&linha.nup);
        previa.resultado = ResultadoRelacionar::SemEfeito;
        return Ok(previa);
    }
    // O AJAX de consulta é um POST de LEITURA, não uma gravação da relação.
    let codificacao = codificacao_da_tela(&mime)
        .ok_or(FalhaRelacionar::CharsetDaTelaDeRelacionamentosNaoSuportado)?;
    let tela = TelaServida {
        url: link.as_str(),
        html: &html,
        codificacao,
    };
    let url = url_complementar(&html)
        .ok_or(FalhaRelacionar::ConsultaComplementarDeRelacionamentoNaoDeclarada)?;
    let consulta = derivar_consulta_ajax(
        &CONSULTA,
        &instalacao,
        &tela,
        &url,
        &[("idProcedimento", pedido.com.as_str())],
    )
    .map_err(FalhaRelacionar::ConsultaDerivacao)?;
    let resposta = rede
        .post(consulta, &cookie)
        .map_err(FalhaRelacionar::ConsultaTransporte)?;
    if resposta.resposta.status_code == 302 {
        return Err(FalhaRelacionar::ConsultaDeRelacionamentoRedirecionada);
    }
    let xml = xml_da_consulta(resposta.resposta)?;
    let DestinoConsultado {
        id: destino_id,
        nup,
        tipo,
    } = ler_destino(&xml)?;
    if destino_id == id {
        return Err(FalhaRelacionar::RelacionamentoConsigoMesmo);
    }
    if !mesmo_protocolo(pedido.com.as_str(), &nup) {
        return Err(FalhaRelacionar::DestinoDaConsultaDivergente);
    }
    // O mesmo id não pode identificar outro NUP na tabela já servida.
    // Caso contrário, "sem efeito" esconderia duas identidades incompatíveis.
    previa.resultado = match relacionados.iter().find(|linha| linha.id == destino_id) {
        Some(linha) if mesmo_protocolo(&linha.nup, &nup) => ResultadoRelacionar::SemEfeito,
        Some(_) => return Err(FalhaRelacionar::DestinoDaConsultaDivergente),
        None => ResultadoRelacionar::Previa,
    };
    previa.id_destino = Some(destino_id);
    previa.tipo_destino = tipo;
    previa.destino = nup;
    previa.requisicoes = rede.feitas();
    if pedido.confirmar && !previa.sem_efeito() {
        escrita::adicionar(
            sessao,
            &instalacao,
            &cookie,
            link.as_str(),
            &html,
            &mime,
            previa
                .id_destino
                .as_deref()
                .ok_or(FalhaRelacionar::DestinoARelacionarNaoIdentificado)?,
            &previa.destino,
            previa.tipo_destino.as_deref().unwrap_or(""),
            &relacionados,
            &mut rede,
        )?;
        previa.resultado = ResultadoRelacionar::Confirmado;
        previa.requisicoes = rede.feitas();
    }
    Ok(previa)
}

impl From<CookieIlegivel> for FalhaRelacionar {
    fn from(erro: CookieIlegivel) -> Self {
        Self::Cookie(erro)
    }
}
