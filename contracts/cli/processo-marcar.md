---
type: CommandContract
title: Atribuir e retirar marcador de processo pela interface Web
operation: processo.marcar
command: ies processo marcar NUP --marcador ID#NOME|--retirar ID#NOME [--texto T] [--confirmar]
effect: write-process-state
implementation: partial
---

# `ies processo marcar` (Refs #255)

A identidade é `id#nome`: o **id** decimal é a identidade observada do
marcador da unidade, e o **nome** confere sua semântica. Apenas a primeira
cerquilha é separadora. O IES rejeita nome vazio e controles e nunca
publica URL assinada, cookie ou `infra_hash`.

## Fatia funcional implementada

- Entrada `NUP` e `id#nome` inválida sai 2 **antes de ler a sessão**.
  Exige exatamente `--marcador` ou `--retirar`; `--texto` só com atribuição e respeita o limite de 250 caracteres do textarea observado. Aceita múltiplas linhas e não presume Latin-1; a codificação pertence ao formulário servido quando a escrita for implementada.
- Usa a leitura Web existente do processo, **sem andamentos e propriedades**,
  mas com os marcadores; exige a unidade publicada pela tela, ignorando
  o atalho que não permite conferi-la.
- Exige a lista completa: a contagem de linhas observadas deve ser igual
  ao `total_declarado`. `null`, link ausente ou tela não reconhecida
  **não** significam lista vazia.
- Se o id está presente e o nome confere, atribuir é idempotente: saída 0,
  `alterado: false`, **sem POST**.
- Se o id não consta da lista completa, retirar também é idempotente,
  saída 0, sem escrita. Se o mesmo id tem nome diferente, recusa.
- Se está presente e o pedido é retirar, produz **prévia** com a unidade,
  id, nome e `requisicoes` realmente contadas, sem POST. Em origem autorizada
  de treinamento, `--confirmar` só envia uma remoção quando observa os
  campos ocultos e o `action` de `acaoRemover` no formulário servido.
  Falta de formulário comprovado recusa sem POST.
- Se está ausente e o pedido é atribuir, consulta o formulário
  `andamento_marcador_cadastrar` oferecido pela unidade. Somente quando
  esse formulário publica exatamente uma opção `id#nome`, a ação assinada
  pertence ao mesmo processo e os controles de submissão estão presentes,
  apresenta a **prévia de atribuição** com `alterado: false` e **zero POST**.
  Se não há catálogo observado ou a opção não confere, recusa
  `CatalogoDaUnidadeNaoObservado`. Um marcador aplicado em outro
  processo **não** substitui o catálogo da unidade.

A prévia não reivindica uma escrita. A confirmação de **atribuição** exige
origem de treinamento, catálogo e formulário da unidade conferidos, **um POST**
e aceite causal por 302 do mesmo processo. Só declara sucesso após nova leitura
independente confirmar o mesmo `id#nome` e o texto solicitado. Uma resposta
incerta, inclusive 403 depois de POST efetivamente aplicado, resulta em
**saída 1**, sem repetir o envio.
A confirmação de **retirada** exige origem de treinamento autorizada,
formulário específico observado e um único POST. Só declara sucesso se o
redirecionamento 302 identifica a operação de retirada no mesmo processo
**e** uma consulta independente demonstra ausência do marcador. Falha na
prova causal ou na releitura após o envio resulta em **saída 1**, sem nova
tentativa, com a orientação de não repetir a confirmação. O código não
executa JavaScript e não classifica erros pelo texto da mensagem.

## Orçamento e sequência da fatia

`ies processo NUP --sem-andamentos --sem-propriedades` com exigência da
unidade e leitura de marcadores. Requisições variam conforme abertura e
cache, mas são contadas pelo leitor. A prévia, idempotência e recusa não
escrevem. Para retirada confirmada, a CLI lê a tela de gerenciamento,
envia no máximo **um POST** e relê o processo independentemente. O teste `previa_de_retirada_e_confirmacao_bloqueada_usam_quatro_get_zero_post`
usa uma abertura fria sintética pelo Controle: **3 GET para abrir o
processo + 1 GET para a tela dos marcadores = 4 GET, 0 POST**, tanto
na prévia quanto na confirmação recusada. O transporte roteirizado
confere a lista exata de pedidos, sem rede real. A contagem de outros
caminhos de abertura e do índice quente precisa de novos casos.
Testes sintéticos também conferem idempotência, lista parcial,
identidade divergente e texto Latin-1 válido. O E2E da prévia de retirada
presente usa a CLI Rust real, HTTPS local, anti-IES e SQLite efêmero,
compara saída humana/JSON e consulta novamente os marcadores para
comprovar **estado inalterado e zero POST**. Na **prévia**, o formulário de remoção ainda não foi validado para envio:
`retirada_possivel` permanece `false`, mesmo com marcador presente. Isso
não significa que a capacidade de retirada esteja indisponível. A saída
humana deixa explícito que a prévia não alterou o processo e que uma tentativa
de `--confirmar` somente é admitida em treinamento **se** o formulário
servido for conferido. A segunda metade do BDD-255-02 já possui E2E
sintético com POST único e releitura, sem prova externa de equivalência.
Entrada inválida não lê configuração.

## Inventário normativo de subcódigos

A tabela abaixo descreve os erros; o stderr traz o nome do erro.
Recusas pré-submissão permanecem **sem POST**. Se o POST chegou a ser tentado,
a confirmação depende da resposta causal e da releitura independente.

| Erro (stderr) | Condição observável ou regra |
| --- | --- |
| `ErroMarcar::RotuloSemSeparador` | Rótulo sem `#` |
| `IdDoMarcadorNaoDecimal` | Identificador ausente ou fora da forma decimal |
| `NomeDoMarcadorVazio` | Nome vazio após o separador |
| `NomeDoMarcadorComControle` | Caracteres de controle no nome |
| `ListaDeMarcadoresParcial` | Linhas observadas diferentes do total declarado |
| `RotuloDiferenteDoMarcadorLido` | Mesmo id observado com nome diferente |
| `CatalogoDaUnidadeNaoObservado` | Atribuição ausente sem catálogo da unidade comprovado |
| `RemocaoSemFormularioObservado` | Remoção sem ação/controles observados no formulário servido |
| `ErroMarcar::MarcadoresNaoLidos` | Leitura ausente ou não comprovada |
| `ErroMarcar::UnidadeDaSessaoNaoPublicada` | Unidade não conferida |
| `TextoDoMarcadorLongo` | Texto excede 250 caracteres |

## Falhas tipadas

Entrada (2): `numero_processo_vazio`, `numero_processo_controle`,
`referencia_processo_legada`, `marcador_nao_informado`,
`ErroMarcar::RotuloSemSeparador`, `IdDoMarcadorNaoDecimal`,
`NomeDoMarcadorVazio`, `NomeDoMarcadorComControle`,
`TextoDoMarcadorLongo`.

Leitura/decisão (1): erros específicos da abertura do processo, além de
`ErroMarcar::MarcadoresNaoLidos`,
`ErroMarcar::UnidadeDaSessaoNaoPublicada`,
`ListaDeMarcadoresParcial`,
`RotuloDiferenteDoMarcadorLido`,
`CatalogoDaUnidadeNaoObservado`,
`RemocaoSemFormularioObservado`.

O `status_http` e a mensagem saneada do SEI são preservados pela
camada de leitura existente. A saída de prévia JSON traz
`nup`, `id_procedimento`, `unidade`, `marcador`, `acao`,
`confirmado`, `alterado: false`, `nada_a_fazer`,
`retirada_possivel` e `requisicoes`.

## Ainda faltam (sem promessa de escrita)

Restam o confronto semântico bilateral com as cassettes de **atribuição e remoção**,
prévia e escrita live em UnB
e nas demais instalações autorizadas, incluindo resíduo e orçamento. O
POST único de retirada e sua releitura estão implementados **somente com
prova sintética** pela CLI real ↔ anti-IES ↔ SQLite; a equivalência com
uma instalação SEI ainda não foi comprovada. A prévia de atribuição usa formulário do catálogo independente no anti-IES;
a confirmação única e a releitura foram exercitadas sinteticamente, não no SEI externo.
Escritas são autorizadas apenas em **treinamento**, com prévia, um único
envio e verificação por leitura, nunca retry.

Ver [estrutura observada](../../docs/como-o-sei-funciona/marcadores-do-processo.md),
[decisão](../../docs/decisoes/processo-marcar.md) e
[relatório live](../../docs/validacao-live/2026-11-05-processo-marcar.md).
