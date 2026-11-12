"""Ajustes à mão nos contratos e na documentação que citam os nomes antigos."""

from ajustes_base import ajuste, trocar


# --- contratos e documentação que citam os nomes antigos ---------------------


@ajuste("contracts/cli.md", "o exemplo de erro sem sessão é o novo Debug")
def _(t):
    return trocar(
        t,
        "(por exemplo `ArquivoPrivadoAusenteOuInacessivel`)",
        "(por exemplo `UrlDoControleIlegivel(Ausente)`)",
    )


@ajuste(
    "contracts/cli/sessao-login.md",
    "os erros de arquivo do login passam a ter nome e ErroArquivoPrivado",
)
def _(t):
    return trocar(
        t,
        """`LinhaDasCredenciaisSemCampoConhecido`, os de arquivo privado
(`arquivo_privado_*`), `LoginEmAndamentoEmOutroProcesso`,""",
        """`LinhaDasCredenciaisSemCampoConhecido`, os de arquivo privado
(`ArquivoDeCredenciais`, `OrigemIlegivel`, `ArquivoDaSessao`, cada um com
o `ErroArquivoPrivado` do arquivo), `OrigemNaoAutorizada`,
`LoginEmAndamentoEmOutroProcesso`,""",
    )


@ajuste(
    "docs/arquitetura/0005-erros-por-tipo.md",
    "os quatro tipos de uma variante só que embrulhavam arquivo saíram",
)
def _(t):
    return trocar(
        t,
        """  `From` (`CookieIlegivel`, `CredenciaisIlegiveis`, `OrigemIlegivel`,
  `SessaoGravadaInvalida`, `HistoricoNaoPedido`, `ErroDasAnotacoes`).
  O erro `transparente` mantém o motivo concreto como nome público.
""",
        """  `From` (`HistoricoNaoPedido`, `ErroDasAnotacoes`).
  O erro `transparente` mantém o motivo concreto como nome público.
- **Arquivo privado fora do transporte** (12/11/2026): `CookieIlegivel`,
  `CredenciaisIlegiveis`, `OrigemIlegivel` e `SessaoGravadaInvalida`
  tinham uma variante só, que embrulhava a `FalhaTransporte` de uma
  leitura de arquivo. A leitura (`arquivo_privado::ler`) devolve
  `ErroArquivoPrivado`, um fato por variante, e quem lê embrulha uma vez
  com o nome do arquivo: `FalhaTransporte::CookieDaSessaoIlegivel`,
  `ErroLogin::ArquivoDeCredenciais`. Os comandos recebem o cookie que a
  abertura leu e conferiu (`CookieLido`, em `AcoesDeEstado`,
  `DocumentoNaArvoreAberto`, `DocumentoObtido` e `Troca`): a variante
  `Cookie` dos comandos, que nenhum caminho produzia, saiu. Só `processo
  criar` lê o cookie primeiro, e cada fato do retrato é variante de
  `ErroCriar` (`CookieDaSessaoIlegivel`, `CookieForaDoFormato`,
  `ControleForaDaOrigem`).
""",
    )


@ajuste(
    "docs/decisoes/arquivos-privados/gravacao-privada.md",
    "a leitura mudou de módulo: arquivo_privado::ler",
)
def _(t):
    t = trocar(
        t,
        """  subcódigo. Os módulos continuam donos da serialização, dos nomes e dos
""",
        """  subcódigo. Desde 12/11/2026 a leitura mora no mesmo módulo
  (`arquivo_privado::ler`, com `ErroArquivoPrivado`). Os módulos
  continuam donos da serialização, dos nomes e dos
""",
    )
    return trocar(
        t,
        """origem e o marcador `.geracao`) continuam lidos por
`transporte::arquivo_privado`, que recusa link no nome final; as outras""",
        """origem e o marcador `.geracao`) continuam lidos por
`arquivo_privado::ler`, que recusa link no nome final; as outras""",
    )


@ajuste(
    "docs/erros-tipados/locais.md",
    "o link no nome final é a variante LinkSimbolico de ErroArquivoPrivado",
)
def _(t):
    return trocar(
        t, "(`arquivo_privado_link_simbolico`).", "(`ErroArquivoPrivado::LinkSimbolico`)."
    )
