---
type: CommandContract
title: Prévia e salvamento seguro de documento
operation: documento.salvar
command: ies documento salvar DOC
implementation: partial
effect: write
---

# Prévia local e salvamento de documento

`ies documento salvar DOC [--versao N] [--confirmar]` consulta a cópia
local versionada no diretório atual. Sem `--versao`, seleciona a maior
versão **local** observada; isso não afirma a versão remota atual.

Uma cópia segura exige `DOC.vN.html`, `.DOC.vN.base.html` e
`.DOC.vN.meta.json`, com identidade de instalação, ID do documento,
versão positiva e hash SHA-256 da base. O arquivo da visualização
histórica (`DOC/versao-N.html`) nunca é uma cópia de trabalho.

A prévia é **offline**, compara a cópia com a base, não expõe o corpo,
URLs assinadas ou cookies, e não abre o editor. Falhas de base,
identidade, links simbólicos ou inconsistências têm subcódigos tipados.

`--confirmar` **não envia POST no SEI**: retorna
`escrita_remota_nao_validada` após conferir a cópia. A escrita Web,
mesclagem por seção, concorrência, assinatura e promoção para uma nova
versão dependem de validação específica por instalação; não podem
ser simuladas. O ciclo completo de #89 permanece pendente.

- [Evidências legadas](../evidencias/documento-salvar.md)
- [Decisão arquitetural](../../docs/decisoes/documento-salvar.md)
- [Testes de CLI](../../tests/cli_contract.rs)
