---
type: ImplementationDecision
title: Leitura do prazo a partir do link assinado da barra e da tela observada
operation: processo.prazo
implementation: partial
status: adopted-for-scope
scope: leitura e prévia Web; confirmação sintética por data ou dias somente no debug local; exclusão fechada
selected_strategy: web-link-da-barra-formulario-estrito
alternatives: web-link-da-barra-formulario-estrito,ler-pela-lista-da-unidade,rest-wssei
evidence_level: cassettes-publicas-unb-go-ufrj-e-testes-sinteticos
live_validation: not-run
---

# Controle de prazo (Refs #272)

## Contexto

A barra do nó `PROCESSO` publica `controle_prazo_definir`;
a tela autentica o processo por seu `hdnIdProcedimentos` e pela ação
do formulário.

## Alternativas consideradas

- `web-link-da-barra-formulario-estrito`: ler o link assinado da barra e
  a tela servida, recusando formas desconhecidas.
- `ler-pela-lista-da-unidade`: partir de `controle_prazo_listar`.
- `rest-wssei`: usar o serviço REST/WSSEI.

## Comparação

A alternativa da lista da unidade é indispensável para **exclusão**:
a barra não a oferece; a lista `controle_prazo_listar` usa
`acaoExcluir` e `controle_prazo_excluir`. Portanto não seria seguro
fingir que a exclusão se faz pelo formulário da barra. REST/WSSEI não
tem observação que fundamente sua seleção.

## Decisão

Foi escolhida a **leitura Web estrita** antes de qualquer
POST. O leitor recusa formas desconhecidas, não faz `retry`, não
normaliza URLs assinadas e usa `html_atributos`.

A confirmação por data ou dias foi implementada **somente no binário
debug com origem reservada `https://sei.test/sei`**. Envia um único
POST, exige resposta 302 compatível e relê o prazo exato no mesmo
SQLite do anti-IES; uma resposta inconclusiva produz saída 1 e nenhum
retry. O binário release recusa toda confirmação antes da rede,
assim como a exclusão não implementada. Essa emulação não comprova
o comportamento do SEI externo.

## Evidências verificáveis

- [Contrato](../../contracts/cli/processo-prazo.md) e
  [matriz de evidências](../../contracts/evidencias/processo-prazo.md).
- [Teste Rust de superfície](../../tests/cli_processo_prazo.rs),
  [E2E da CLI real com SQLite](../../tests/aies_prazo_cli.rs) e
  [relatório live](../validacao-live/2026-11-05-processo-prazo.md).
- Os testes sintéticos derivam dos casos de
  `franklinbaldo/todos@323eb01:tests/test_ciclo_prazo_010.py`
  (somente cenários) e verificam ano bissexto, exclusividade, leitura
  dos estados, formulário alterado, id divergente e limpeza do campo
  incompatível ao preparar a prévia.

## Limitações e revisão

O estado por dias (`rdoPrazo=2`) é representado como contagem
tipada, não como data deduzida. A validação sintética da CLI não substitui
confronto semântico independente com os ZIPs UnB/GO, ainda NOT_RUN;
o treinamento também continua NOT_RUN e, se executado, deve registrar
orçamento, reconciliação e resíduo. Exclusão depende da lista da unidade,
não do formulário de definição.
