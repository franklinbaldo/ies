---
type: CommandContract
title: Controle de prazo do processo
operation: processo.prazo
command: ies processo prazo NUP [--ate DD/MM/AAAA | --dias N | --excluir] [--confirmar]
effect: read-and-preview-write-process-state
implementation: partial
---

# `ies processo prazo NUP` (Refs #272)

Lê a situação do controle de prazo pela tela Web oferecida na barra do
processo. `--ate` e `--dias` produzem uma **prévia** sem escrita.
Na **CLI de desenvolvimento**, exclusivamente com a origem reservada
`https://sei.test/sei`, `--confirmar` envia uma única requisição POST
para o anti-IES e relê o formulário antes de declarar sucesso. No binário
de distribuição, a confirmação permanece bloqueada antes de qualquer
requisição. `--excluir` deriva da lista de prazos da unidade (`controle_prazo_listar`),
com envio unico de `controle_prazo_excluir` e verificacao por releitura.
A escrita de exclusao so roda em debug com a origem sintetica reservada.

## Entrada e saída

- `NUP`: validado antes dos arquivos; uso inválido é saída 2.
- `--ate DD/MM/AAAA`: data certa, calendário gregoriano e ano positivo.
- `--dias N`: 1 a 999 dias corridos; a contagem é do SEI.
- `--excluir`: exclusivo com os dois anteriores.
- `--confirmar`: recusado antes da rede no binário release; no binário
  debug, aceito para `--ate`/`--dias` e `--excluir` somente na origem sintetica
  reservada, com um POST e releitura independente.
- `--unidade`: conferida na abertura pela tela do processo.
- Saída JSON: `nup`, `id_procedimento`, `unidade`, `origem=web`,
  `atual` (`sem_prazo`, `data`, `dias` ou `concluido`), `solicitado`,
  `campos_da_previa`, `confirmado` e `requisicoes`. A confirmação só
  marca `true` após GET novo comprovar exatamente o prazo solicitado.

## Caminho e orçamento

1. Abrir o processo por `ies processo NUP`, com unidade exigida e
   sem andamentos/propriedades/marcadores. O número de GETs varia
   conforme o índice, como no contrato de abertura.
2. A barra do nó `PROCESSO` precisa publicar um único link assinado
   `controle_prazo_definir`, com `id_procedimento` igual ao aberto.
3. Mais **um GET** da tela assinada; nenhum POST em leitura ou prévia.
4. Ler exclusivamente o formulário `frmControlePrazoCadastro`, método
   POST, ação e id do processo conferidos, ocultos de identidade,
   `sbmDefinirControlePrazo`, `rdoPrazo` (data 1, dias 2,
   opcionalmente concluir 3), `txtPrazo`, `txtDias`,
   `chkSinDiasUteis`. Controle duplicado, desabilitado, alterado
   ou de outro processo: falha fechada. Atributos via
   `html_atributos`, nunca executar JavaScript.
5. A prévia para dias limpa `txtPrazo`, e a prévia por data limpa
   `txtDias`; não inclui `chkSinDiasUteis` sem pedido de úteis.

Orçamento esperado: **abertura + 1 GET** para leitura e prévia;
na confirmação sintética, **abertura + GET do formulário + 1 POST
+ 1 GET de verificação**. Requisições posteriores à escrita verificam o
estado persistido no mesmo SQLite. Uso inválido, exclusão fora do simulador e confirmação
em release resultam em **zero requisições**. Uma resposta inconclusiva
após o POST retorna código 1 sem retry, redirecionamento seguido ou
restauração de tela.

## Falhas e subcódigos

| Erro (stderr) | Condição |
| --- | --- |
| `ErroPrazo::OpcoesIncompativeis` | Combinação de opções mutuamente exclusivas |
| `ErroPrazo::DataInvalida` | Data não existe |
| `ErroPrazo::DiasInvalidos` | Contagem fora da faixa |
| `ErroPrazo::FormularioDeOutroDestino` | O `action` do formulário não é `controle_prazo_definir` em `controlador.php` |
| `ErroPrazo::IdDoProcessoAusenteNoFormulario` | O formulário não traz `hdnIdProcedimentos` |
| `ErroPrazo::OpcoesDoPrazoAusentes` | O formulário não publica as opções `rdoPrazo` |
| `ErroPrazo::PrazoAtualIlegivel` | A opção marcada ou os dias não formam um prazo |
| `ErroPrazo::OutroProcesso` | Identidade distinta |
| `ErroLinkPrazo::BarraNaoInterpretada` | Barra sem leitura segura |
| `ErroLinkPrazo::LinksRepetidos` | Múltiplas ações de prazo |
| `ErroLinkPrazo::EnderecoNaoReconhecido` | URL fora da forma segura |
| `ErroLinkPrazo::EnderecoDeOutroProcesso` | URL de outro processo |
| `FalhaDoPrazo::ControleNaoOferecidoNaBarra` | Ação ausente |
| erro do clap (saída 2) | `--confirmar` sem `--ate`, `--dias` ou `--excluir` (uso, saída 2) |

Falhas do transporte e da abertura preservam os subcódigos específicos das
respectivas peças; não se publicam cookies nem URLs assinadas.

## Evidências e pendências

A forma Web está documentada em `Todos`
`tests/test_ciclo_prazo_010.py @323eb01` e nos índices imutáveis
`cassettes/unb/processo-prazo-unb--48614e412f28.md` e
`cassettes/go/processo-prazo-go--4d9b8462dae9.md`.
Aproveitam-se os cenários, não a arquitetura Python.

**Cobertura sintética de BDD-272-02:** testes em
[`aies_prazo_cli.rs`](../../tests/aies_prazo_cli.rs) executam a CLI real
por TLS loopback até o anti-IES e SQLite v21, com POST único e
verificação por GET para data e número de dias. A resposta 403 após
persistir a data produz código 1, sem repetição. O tipo `dias` preserva
a contagem observada sem inventar data de calendário. A integridade dos
ZIPs UnB/GO foi verificada separadamente; **paridade semântica com as
respostas originais e treinamento continuam NOT_RUN**.

**BDD-272-03 (hipotese sintetica):** o anti-IES serve a lista da unidade
com controles ativos derivados do SQLite, e a CLI so envia o POST apos
selecionar a linha pelo NUP. O id do item da lista e hipoteticamente igual
ao id do processo no simulador, enquanto as capturas reais podem distingui-los.
A lista e o formulario de definicao sao relidos apos o POST; falha ou efeito
inconclusivo resulta em `FalhaDoPrazo::NaoConfirmado`, saida 1, sem retry. A integridade das cassettes de
exclusao UnB/UFRJ/PEN3 foi verificada em Actions #42633542261 (73 interacoes,
POST controle_prazo_excluir em cada uma). Paridade semantica e live NOT_RUN.

**Ainda falta para concluir #272:** confronto independente do acervo e
validacao live em treinamento autorizado. Nunca escrever na producao.
